pub const ADDI: u8 = 0;
pub const SUBI: u8 = 1;
pub const MULI: u8 = 2;
pub const DIVI: u8 = 3;
pub const GEQI: u8 = 4;
pub const GRI: u8 = 5;
pub const LEQI: u8 = 6;
pub const LESQI: u8 = 7;
pub const EQI: u8 = 8;
pub const NEI: u8 = 9;
pub const ADDR: u8 = 10;
pub const SUBR: u8 = 11;
pub const MULR: u8 = 12;
pub const DIVR: u8 = 13;
pub const GEQR: u8 = 14;
pub const GRR: u8 = 15;
pub const LEQR: u8 = 16;
pub const LESQR: u8 = 17;
pub const EQR: u8 = 18;
pub const NER: u8 = 19;
pub const CSTI: u8 = 20;
pub const CSTR: u8 = 21;
pub const OR: u8 = 22;
pub const AND: u8 = 23;
pub const RDI: u8 = 24; // 24 % 4 = 0
pub const RDR: u8 = 25; // 25 % 4 = 1
pub const RDB: u8 = 26; // 26 % 4 = 2
pub const RDS: u8 = 27; // 27 % 4 = 3
pub const WRI: u8 = 28; // 28 % 4 = 0
pub const WRR: u8 = 29; // 29 % 4 = 1
pub const WRB: u8 = 30; // 30 % 4 = 2
pub const WRS: u8 = 31; // 31 % 4 = 3
pub const WRLI: u8 = 32; // 32 % 4 = 0
pub const WRLR: u8 = 33; // 33 % 4 = 1
pub const WRLB: u8 = 34; // 34 % 4 = 2
pub const WRLS: u8 = 35; // 35 % 4 = 3
pub const LDI: u8 = 36; // 36 % 4 = 0
pub const LDR: u8 = 37; // 37 % 4 = 1
pub const LDB: u8 = 38; // 38 % 4 = 2
pub const LDS: u8 = 39; // 39 % 4 = 3
pub const STRI: u8 = 40; // 40 % 4 = 0
pub const STRR: u8 = 41; // 41 % 4 = 1
pub const STRB: u8 = 42; // 42 % 4 = 2
pub const STRS: u8 = 43; // 43 % 4 = 3
pub const JUMP: u8 = 44;
pub const JEQ: u8 = 45;
pub const JNE: u8 = 46;
pub const LBL: u8 = 47;
pub const CALL: u8 = 48;
pub const RET: u8 = 49;
pub const EXT: u8 = 50;
pub const LDIC: u8 = 51; // 51 % 4 = 3
pub const LDRC: u8 = 52; // 52 % 4 = 0
pub const LDBC: u8 = 53; // 53 % 4 = 1
pub const LDSC: u8 = 54; // 54 % 4 = 2
pub const PARAM: u8 = 55;
pub const STRIP: u8 = 56; // 56 % 4 = 0
pub const STRRP: u8 = 57; // 57 % 4 = 1
pub const STRBP: u8 = 58; // 58 % 4 = 2
pub const STRSP: u8 = 59; // 59 % 4 = 3
pub const FUNC: u8 = 60;
