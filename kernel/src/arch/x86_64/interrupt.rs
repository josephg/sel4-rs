
const IRQ_INT_OFFSET: u8 = 0x20;
//
// #[repr(u8)]
// pub enum Interrupt {
//     Debug = 1,
//     SoftwareBreakRequest = 3,
//     UnimplDev = 7,
//     GPFault = 13,
//     PageFault = 14,
//
//     /// First IRQ.
//     IrqMin = IRQ_INT_OFFSET,
//     // IrqIsaMin = IRQ_INT_OFFSET,
//     IrqIsaMax =
// }