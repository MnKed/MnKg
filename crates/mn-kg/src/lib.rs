use rand::{thread_rng, Rng};
use sha1::Digest;
use strum::{AsRefStr, EnumIter, Display};

pub use strum::IntoEnumIterator;

#[repr(u8)]
#[derive(Display, Clone, Copy, AsRefStr, EnumIter, Debug, PartialEq, Eq)]
pub enum App {
    FocusV3 = 0xE,
    FocusV3Pro = 0xF,
    GraffleV7 = 9,
    GraffleV7Pro = 8,
    OutlinerV5 = 0xC,
    OutlinerV5Pro = 0xD,
    PlanV3 = 0x6,
    PlanV3Pro = 0x7,
}

pub fn generate_serial(username: &str, app: App) -> Option<String> {
    if username.is_empty() {
        return None;
    }
    let joined_name: String = username
        .to_string()
        .chars()
        .filter(|it| it.is_ascii_alphanumeric())
        .collect();

    let mut vec2 = [0u8; 13];
    unsafe {
        for i in 5..=9 {
            // SAFETY: Elements in 5..9 < 13
            *vec2.get_unchecked_mut(i) = thread_rng().gen::<u8>();
        }
        // SAFETY: Elements in 10 < 13
        *vec2.get_unchecked_mut(10) = thread_rng().gen_range(0..=3);
        for i in 11..=12 {
            // SAFETY: Elements in 11..=12 < 13
            *vec2.get_unchecked_mut(i) = 0xFF;
        }
    }

    let mut obf = obf_to_ascii(&vec2[5..13]).unwrap();
    for idx in [12, 8, 4] {
        obf.insert(idx, '-')
    }
    let salted = format!("{}{}{}", APP_UID_LIST[app as usize], obf, joined_name);

    let mut vec1 = {
        let mut sha1 = sha1::Sha1::new();
        sha1.update(salted.as_bytes());
        sha1.finalize().to_vec()
    };

    let offset = 20 * (app as usize);
    for i in 0..20 {
        vec1[i] ^= APP_CONST_LIST[offset + i];
    }

    for i in 0..5 {
        vec1[i] ^= vec1[i + 15] ^ vec1[i + 10] ^ vec1[i + 5]
    }

    vec2[..5].copy_from_slice(&vec1[..5]);

    for (i, ele) in vec2.iter_mut().enumerate() {
        *ele = !(*ele ^ joined_name.as_bytes()[i % joined_name.len()]);
    }

    let mut obf2 = obf_to_ascii(&vec2).unwrap();

    for idx in [24, 20, 16, 12, 8, 4] {
        obf2.insert(idx, '-')
    }

    obf2.remove(obf2.len() - 1);

    Some(obf2)
}

const APP_UID_LIST: &[u32; 20] = &[
    1000205, 1000216, 1000215, 1000200, 1000211, 1000210, 1000219, 1000220, 1000221, 1000222,
    1000212, 1000209, 1000224, 1000223, 1000228, 1000226, 1000214, 1000208, 1000207, 1000197,
];

const APP_CONST_LIST: &[u8; 400] = include_bytes!("./apps.bin");

fn obf_to_ascii(data: &[u8]) -> Option<String> {
    if data.is_empty() {
        return None;
    }

    let mut tmp_i = 0;
    let mut tmp_v: u64 = 0;

    let mut buf = [0u8; 48];

    let mut result = Vec::new();

    for ele in data {
        tmp_v = *ele as u64 | ((tmp_v << 8) & 0xFFFFFFFF);

        if tmp_i == 3 {
            let mut sbuff = [0u8; 8];
            for i in (0..=7).rev() {
                let rdx = (0x4EC4EC4EC4EC4EC5u128
                    .overflowing_mul(tmp_v as u128)
                    .0
                    .overflowing_shr(64)
                    .0)
                    .overflowing_shr(3)
                    .0 as u64;
                let eax = (rdx.overflowing_mul(5).0) as u32;
                let eax = eax.overflowing_mul(5).0;
                let eax = ((eax as u64).overflowing_add(rdx).0) as u32;
                sbuff[7 - i] = (tmp_v.overflowing_sub(eax as u64).0) as u8;
                tmp_v = rdx;
            }

            for i in 0..=6 {
                buf[i] = sbuff[6 - i].overflowing_add(65).0;
            }

            for ele in buf {
                if ele == 0 {
                    break;
                }
                result.push(ele);
            }

            tmp_i = 0;
            tmp_v = 0;
        } else {
            tmp_i += 1;
        }
    }

    if tmp_i > 0 {
        const ARR: [usize; 4] = [2, 4, 6, 7];
        let mut sbuff = [0u8; 8];
        let j = ARR[tmp_i - 1];

        for ele in sbuff.iter_mut().take(j + 1) {
            let rdx = (0x4EC4EC4EC4EC4EC5u128
                .overflowing_mul(tmp_v as u128)
                .0
                .overflowing_shr(64)
                .0)
                .overflowing_shr(3)
                .0 as u64;
            let eax = (rdx.overflowing_mul(5).0) as u32;
            let eax = eax.overflowing_mul(5).0;
            let eax = ((eax as u64).overflowing_add(rdx).0) as u32;
            *ele = (tmp_v.overflowing_sub(eax as u64).0) as u8;
            tmp_v = rdx;
        }

        for i in 0..j {
            buf[i] = sbuff[j - i - 1].overflowing_add(65).0;
        }

        for ele in buf {
            if ele == 0 {
                break;
            }
            result.push(ele);
        }
    }

    Some(String::from_utf8(result).unwrap())
}
