use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::num::NonZeroUsize;
use gdbstub::arch;
use gdbstub::arch::{Arch, lldb, Registers};
use gdbstub::common::Signal;
use gdbstub::conn::Connection;
use gdbstub::stub::run_blocking::{BlockingEventLoop, Event, WaitForStopReasonError};
use gdbstub::stub::SingleThreadStopReason;
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::base::single_register_access::{SingleRegisterAccess, SingleRegisterAccessOps};
use gdbstub::target::ext::base::singlethread::{SingleThreadBase, SingleThreadRangeSteppingOps, SingleThreadResume, SingleThreadResumeOps, SingleThreadSingleStepOps};
use gdbstub::target::ext::breakpoints::{Breakpoints, BreakpointsOps, HwWatchpoint, HwWatchpointOps, SwBreakpoint, SwBreakpointOps, WatchKind};
use gdbstub::target::ext::memory_map::MemoryMapOps;
use gdbstub::target::ext::monitor_cmd::{ConsoleOutput, MonitorCmd, MonitorCmdOps};
use gdbstub::target::Target;
use gdbstub::target::TargetResult;

use crate::nes::{NES, StatusRegister};

pub(crate) struct Debugger {
    breakpoints: Vec<u16>,
    read_watchpoints: Vec<u16>,
    write_watchpoints: Vec<u16>,
    watchpoint_details: Vec<NesWatchpoint>,
}

#[derive(PartialEq)]
struct NesWatchpoint {
    addr: u16,
    len: u16,
    kind: WatchKind,
}

pub enum MOS6502 {}

impl Arch for MOS6502 {
    type Usize = u16;
    type Registers = Registers6502;
    type BreakpointKind = (); // We only have one kind of breakpoint (unlike eg: ARM mode vs Thumb mode on AArch32)
    type RegId = RegId;

    fn target_description_xml() -> Option<&'static str> {
        None // TODO
    }

    fn lldb_register_info(reg_id: usize) -> Option<lldb::RegisterInfo<'static>> {
        let reg: lldb::Register = match reg_id {
            0 => lldb::Register { name: "PC", alt_name: None, bitsize: 16, offset: 0, encoding: lldb::Encoding::Uint, format: lldb::Format::Hex, set: "main", gcc: None, dwarf: None, generic: Some(lldb::Generic::Pc), container_regs: None, invalidate_regs: None },
            1 => lldb::Register { name: "A", alt_name: None, bitsize: 8, offset: 2, encoding: lldb::Encoding::Uint, format: lldb::Format::Decimal, set: "main", gcc: None, dwarf: None, generic: None, container_regs: None, invalidate_regs: None },
            2 => lldb::Register { name: "X", alt_name: None, bitsize: 8, offset: 3, encoding: lldb::Encoding::Uint, format: lldb::Format::Decimal, set: "main", gcc: None, dwarf: None, generic: None, container_regs: None, invalidate_regs: None },
            3 => lldb::Register { name: "Y", alt_name: None, bitsize: 8, offset: 4, encoding: lldb::Encoding::Uint, format: lldb::Format::Decimal, set: "main", gcc: None, dwarf: None, generic: None, container_regs: None, invalidate_regs: None },
            4 => lldb::Register { name: "SP", alt_name: None, bitsize: 8, offset: 5, encoding: lldb::Encoding::Uint, format: lldb::Format::Decimal, set: "main", gcc: None, dwarf: None, generic: Some(lldb::Generic::Sp), container_regs: None, invalidate_regs: None },
            5 => lldb::Register { name: "SR", alt_name: None, bitsize: 8, offset: 6, encoding: lldb::Encoding::Uint, format: lldb::Format::Hex, set: "main", gcc: None, dwarf: None, generic: Some(lldb::Generic::Flags), container_regs: None, invalidate_regs: None },
            _ => return Some(lldb::RegisterInfo::Done),
        };

        Some(lldb::RegisterInfo::Register(reg))
    }
}

#[derive(Debug)]
pub enum RegId {
    PC,
    A,
    X,
    Y,
    SP,
    SR,
}

impl arch::RegId for RegId {
    fn from_raw_id(id: usize) -> Option<(Self, Option<NonZeroUsize>)> {
        match id {
            0 => Some((RegId::PC, NonZeroUsize::new(2))),
            1 => Some((RegId::A, NonZeroUsize::new(1))),
            2 => Some((RegId::X, NonZeroUsize::new(1))),
            3 => Some((RegId::Y, NonZeroUsize::new(1))),
            4 => Some((RegId::SP, NonZeroUsize::new(1))),
            5 => Some((RegId::SR, NonZeroUsize::new(1))),
            _ => return None,
        }
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Registers6502 {
    PC: u16,
    A: u8,
    X: u8,
    Y: u8,
    SP: u8,
    SR: u8,
}

impl Registers for Registers6502 {
    type ProgramCounter = u16;

    fn pc(&self) -> Self::ProgramCounter {
        self.PC
    }

    fn gdb_serialize(&self, mut write_byte: impl FnMut(Option<u8>)) {
        write_byte(Some(self.PC.to_le_bytes()[0]));
        write_byte(Some(self.PC.to_le_bytes()[1]));
        write_byte(Some(self.A));
        write_byte(Some(self.X));
        write_byte(Some(self.Y));
        write_byte(Some(self.SP));
        write_byte(Some(self.SR));
    }

    fn gdb_deserialize(&mut self, bytes: &[u8]) -> Result<(), ()> {
        self.PC = u16::from_le_bytes(bytes[0..=1].try_into().unwrap());
        self.A = bytes[2];
        self.X = bytes[3];
        self.Y = bytes[4];
        self.SP = bytes[5];
        self.SR = bytes[6];
        Ok(())
    }
}

impl Target for NES {
    type Arch = MOS6502;
    type Error = ();

    fn base_ops(&mut self) -> BaseOps<'_, Self::Arch, Self::Error> {
        BaseOps::SingleThread(self)
    }

    fn support_breakpoints(&mut self) -> Option<BreakpointsOps<'_, Self>> {
        Some(self)
    }

    fn support_monitor_cmd(&mut self) -> Option<MonitorCmdOps<'_, Self>> {
        Some(self)
    }

    fn support_memory_map(&mut self) -> Option<MemoryMapOps<'_, Self>> {
        None // TODO
    }
}

impl Breakpoints for NES {
    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<'_, Self>> {
        Some(self)
    }

    fn support_hw_watchpoint(&mut self) -> Option<HwWatchpointOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadBase for NES {
    fn read_registers(&mut self, regs: &mut <Self::Arch as Arch>::Registers) -> TargetResult<(), Self> {
        regs.PC = self.PC;
        regs.A = self.A;
        regs.X = self.X;
        regs.Y = self.Y;
        regs.SP = self.SP;
        regs.SR = self.SR.to_byte();
        Ok(())
    }

    fn write_registers(&mut self, regs: &<Self::Arch as Arch>::Registers) -> TargetResult<(), Self> {
        self.PC = regs.PC;
        self.A = regs.A;
        self.X = regs.X;
        self.Y = regs.Y;
        self.SP = regs.SP;
        self.SR = StatusRegister::from_byte(regs.SR);
        Ok(())
    }

    fn support_single_register_access(&mut self) -> Option<SingleRegisterAccessOps<'_, (), Self>> {
        Some(self)
    }

    fn read_addrs(&mut self, start_addr: <Self::Arch as Arch>::Usize, data: &mut [u8]) -> TargetResult<usize, Self> {
        for (i, out) in data.iter_mut().enumerate() {
            *out = self.read8_no_tick(start_addr + i as u16);
        }
        Ok(data.len())
    }

    fn write_addrs(&mut self, start_addr: <Self::Arch as Arch>::Usize, data: &[u8]) -> TargetResult<(), Self> {
        for (i, input) in data.iter().enumerate() {
            self.write8_no_tick(start_addr + i as u16, *input);
        }
        Ok(())
    }

    fn support_resume(&mut self) -> Option<SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }
}

impl SingleRegisterAccess<()> for NES {
    fn read_register(&mut self, tid: (), reg_id: <Self::Arch as Arch>::RegId, mut buf: &mut [u8]) -> TargetResult<usize, Self> {
        let num_bytes = match reg_id {
            RegId::PC => buf.write(&self.PC.to_le_bytes()[..])?,
            RegId::A => buf.write(&[self.A])?,
            RegId::X => buf.write(&[self.X])?,
            RegId::Y => buf.write(&[self.Y])?,
            RegId::SP => buf.write(&[self.SP])?,
            RegId::SR => buf.write(&[self.SR.to_byte()])?,
        };
        Ok(num_bytes)
    }

    fn write_register(&mut self, tid: (), reg_id: <Self::Arch as Arch>::RegId, val: &[u8]) -> TargetResult<(), Self> {
        match reg_id {
            RegId::PC => { self.PC = u16::from_le_bytes(val.try_into().unwrap()); }
            RegId::A => { self.A = val[0]; }
            RegId::X => { self.X = val[0]; }
            RegId::Y => { self.Y = val[0]; }
            RegId::SP => { self.SP = val[0]; }
            RegId::SR => { self.SR = StatusRegister::from_byte(val[0]); }
        }
        Ok(())
    }
}

impl SingleThreadResume for NES {
    fn resume(&mut self, signal: Option<Signal>) -> Result<(), Self::Error> {
        todo!()
    }

    fn support_single_step(&mut self) -> Option<SingleThreadSingleStepOps<'_, Self>> {
        None // TODO
    }

    fn support_range_step(&mut self) -> Option<SingleThreadRangeSteppingOps<'_, Self>> {
        None // TODO
    }

    // TODO: Try to implement reverse stepping?
}

impl SwBreakpoint for NES {
    fn add_sw_breakpoint(&mut self, addr: <Self::Arch as Arch>::Usize, kind: <Self::Arch as Arch>::BreakpointKind) -> TargetResult<bool, Self> {
        let dbg = self.debugger.as_mut().unwrap();
        add_if_missing(&mut dbg.breakpoints, addr);
        Ok(true)
    }

    fn remove_sw_breakpoint(&mut self, addr: <Self::Arch as Arch>::Usize, kind: <Self::Arch as Arch>::BreakpointKind) -> TargetResult<bool, Self> {
        let dbg = self.debugger.as_mut().unwrap();

        let found = remove_all(&mut dbg.breakpoints, addr);
        Ok(found)
    }
}

impl HwWatchpoint for NES {
    fn add_hw_watchpoint(&mut self, addr: <Self::Arch as Arch>::Usize, len: <Self::Arch as Arch>::Usize, kind: WatchKind) -> TargetResult<bool, Self> {
        let dbg = self.debugger.as_mut().unwrap();
        add_if_missing(&mut dbg.watchpoint_details, NesWatchpoint { addr, len, kind });
        dbg.refresh_watchpoints();
        Ok(true)
    }

    fn remove_hw_watchpoint(&mut self, addr: <Self::Arch as Arch>::Usize, len: <Self::Arch as Arch>::Usize, kind: WatchKind) -> TargetResult<bool, Self> {
        let dbg = self.debugger.as_mut().unwrap();
        remove_all(&mut dbg.watchpoint_details, NesWatchpoint { addr, len, kind });
        dbg.refresh_watchpoints();
        Ok(true)
    }
}

impl Debugger {
    fn refresh_watchpoints(&mut self) {
        self.read_watchpoints.clear();
        self.write_watchpoints.clear();

        for w in self.watchpoint_details.iter() {
            for target in w.addr..w.addr+w.len {
                if w.kind == WatchKind::Read || w.kind == WatchKind::ReadWrite {
                    add_if_missing(&mut self.read_watchpoints, target);
                }
                if w.kind == WatchKind::ReadWrite || w.kind == WatchKind::Write {
                    add_if_missing(&mut self.write_watchpoints, target);
                }
            }
        }
    }
}

impl MonitorCmd for NES {
    fn handle_monitor_cmd(&mut self, cmd: &[u8], mut out: ConsoleOutput<'_>) -> Result<(), Self::Error> {
        gdbstub::outputln!(out, "Unsupported command");
        Err(())
    }
}

fn add_if_missing<T : PartialEq>(vec: &mut Vec<T>, elem: T) {
    if !vec.iter().any(|it| *it == elem) {
        vec.push(elem)
    }
}

fn remove_all<T : PartialEq>(vec: &mut Vec<T>, elem: T) -> bool {
    let mut found = false;
    while let Some(pos) = vec.iter().position(|it| *it == elem) {
        vec.remove(pos);
        found = true;
    }
    found
}

enum NesBlockingEventLoop {}

impl BlockingEventLoop for NesBlockingEventLoop {
    type Target = NES;
    type Connection = TcpStream;
    type StopReason = SingleThreadStopReason<u16>;

    fn wait_for_stop_reason(target: &mut Self::Target, conn: &mut Self::Connection) -> Result<
        Event<Self::StopReason>,
        WaitForStopReasonError<
            <Self::Target as Target>::Error,
            <Self::Connection as Connection>::Error>>
    {
        todo!()
    }

    fn on_interrupt(target: &mut Self::Target) -> Result<Option<Self::StopReason>, <Self::Target as Target>::Error> {
        todo!()
    }
}

fn run_debug(nes: &mut NES) {
    let listener = TcpListener::bind(("localhost", 7890)).unwrap();
    while let Ok((stream, addr)) = listener.accept() {
        let stub = gdbstub::stub::GdbStub::new(stream);
        let res = stub.run_blocking::<NesBlockingEventLoop>(nes);
        match res {
            Ok(disconnect_reason) => println!("Disconnected because {disconnect_reason:?}"),
            Err(gdb_stub_error) => println!("The GDB Stub ran into an error: {gdb_stub_error:?}"),
        }
    }
}
