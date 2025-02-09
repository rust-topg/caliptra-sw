// Licensed under the Apache-2.0 license

use caliptra_emu_bus::Bus;
use caliptra_emu_types::{RvAddr, RvData, RvSize};
use caliptra_verilated::CaliptraVerilated;
use std::io::Write;

use crate::Output;
use std::env;

// TODO: Make this configurable
const SOC_PAUSER: u32 = 0xffff_ffff;

// How many clock cycles before emitting a TRNG nibble
const TRNG_DELAY: u32 = 4;

pub struct VerilatedApbBus<'a> {
    v: &'a mut CaliptraVerilated,
}
impl<'a> Bus for VerilatedApbBus<'a> {
    fn read(&mut self, _size: RvSize, addr: RvAddr) -> Result<RvData, caliptra_emu_bus::BusError> {
        if addr & 0x3 != 0 {
            return Err(caliptra_emu_bus::BusError::LoadAddrMisaligned);
        }
        Ok(self.v.apb_read_u32(SOC_PAUSER, addr))
    }

    fn write(
        &mut self,
        size: RvSize,
        addr: RvAddr,
        val: RvData,
    ) -> Result<(), caliptra_emu_bus::BusError> {
        if addr & 0x3 != 0 {
            return Err(caliptra_emu_bus::BusError::StoreAddrMisaligned);
        }
        if size != RvSize::Word {
            return Err(caliptra_emu_bus::BusError::StoreAccessFault);
        }
        self.v.apb_write_u32(SOC_PAUSER, addr, val);
        Ok(())
    }
}

pub struct ModelVerilated {
    v: CaliptraVerilated,

    output: Output,
    trace_enabled: bool,

    trng_nibbles: Box<dyn Iterator<Item = u8>>,
    trng_delay_remaining: u32,
}

impl ModelVerilated {
    pub fn start_tracing(&mut self, path: &str, depth: i32) {
        self.v.start_tracing(path, depth).unwrap();
    }
    pub fn stop_tracing(&mut self) {
        self.v.stop_tracing();
    }
}

impl crate::HwModel for ModelVerilated {
    type TBus<'a> = VerilatedApbBus<'a>;

    fn new_unbooted(params: crate::InitParams) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        let output = Output::new(params.log_writer);

        let output_sink = output.sink().clone();

        let generic_load_cb = Box::new(move |v: &CaliptraVerilated, ch: u8| {
            output_sink.set_now(v.total_cycles());
            output_sink.push_uart_char(ch);
        });
        let mut v = CaliptraVerilated::with_generic_load_cb(
            caliptra_verilated::InitArgs {
                security_state: u32::from(params.security_state),
            },
            generic_load_cb,
        );

        v.write_rom_image(params.rom);

        let mut m = ModelVerilated {
            v,
            output,
            trace_enabled: false,

            trng_nibbles: params.trng_nibbles,
            trng_delay_remaining: TRNG_DELAY,
        };

        m.tracing_hint(true);

        m.v.input.cptra_pwrgood = true;
        m.v.next_cycle_high(1);

        m.v.input.cptra_rst_b = true;
        m.v.next_cycle_high(1);

        while !m.v.output.ready_for_fuses {
            m.v.next_cycle_high(1);
        }
        writeln!(m.output().logger(), "ready_for_fuses is high")?;
        Ok(m)
    }

    fn apb_bus(&mut self) -> Self::TBus<'_> {
        VerilatedApbBus { v: &mut self.v }
    }

    fn step(&mut self) {
        if self.v.output.etrng_req {
            if self.trng_delay_remaining == 0 {
                if let Some(val) = self.trng_nibbles.next() {
                    self.v.input.itrng_valid = true;
                    self.v.input.itrng_data = val & 0xf;
                }
                self.trng_delay_remaining = TRNG_DELAY;
            } else {
                self.trng_delay_remaining -= 1;
            }
        }
        self.v.next_cycle_high(1);
        self.v.input.itrng_valid = false;
    }

    fn output(&mut self) -> &mut crate::Output {
        self.output.sink().set_now(self.v.total_cycles());
        &mut self.output
    }

    fn ready_for_fw(&self) -> bool {
        self.v.output.ready_for_fw_push
    }

    fn tracing_hint(&mut self, enable: bool) {
        if self.trace_enabled != enable {
            self.trace_enabled = enable;
            if enable {
                if let Ok(trace_path) = env::var("CPTRA_TRACE_PATH") {
                    self.v.start_tracing(&trace_path, 99).ok();
                }
            } else {
                self.v.stop_tracing();
            }
        }
    }
}
