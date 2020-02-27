
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Action {
    Pop = 1,
    Chi = 2,
    Peng = 3,
    Gang = 4,
    Hu = 5,
    ZiMo = 6,
    QiangJin = 7,
    QingYiSe = 8,
    JinQue = 9,
    JinLong = 10,
    DealBeginCard = 11,
    DealNextCard = 12,
}
