use isvp_sys::*;
use log::error;
use std::io::Cursor;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

pub unsafe fn jpeg_start(tx: watch::Sender<Vec<u8>>, flag: Arc<Mutex<bool>>) -> bool {
    if IMP_Encoder_StartRecvPic(2) < 0 {
        error!("IMP_Encoder_StartRecvPic failed");
        return false;
    }

    loop {
        let shutdown_flag = match flag.lock() {
            Ok(guard) => guard,
            Err(_) => continue,
        };

        if *shutdown_flag {
            break true;
        }
        drop(shutdown_flag);

        if IMP_Encoder_PollingStream(2, 10000) < 0 {
            error!("IMP_Encoder_PollingStream failed");
            continue;
        }

        let mut stream = IMPEncoderStream {
            phyAddr: 0,
            virAddr: 0,
            streamSize: 0,
            pack: std::ptr::null_mut(),
            packCount: 0,
            seq: 0,
            isVI: false,
            __bindgen_anon_1: IMPEncoderStream__bindgen_ty_1 {
                streamInfo: IMPEncoderStreamInfo {
                    iNumBytes: 0,
                    uNumIntra: 0,
                    uNumSkip: 0,
                    uNumCU8x8: 0,
                    uNumCU16x16: 0,
                    uNumCU32x32: 0,
                    uNumCU64x64: 0,
                    iSliceQP: 0,
                    iMinQP: 0,
                    iMaxQP: 0,
                },
            },
        };

        if IMP_Encoder_GetStream(2, &mut stream, true) < 0 {
            error!("IMP_Encoder_GetStream failed");
            continue;
        }

        let stream_packs = std::slice::from_raw_parts(
            stream.pack as *const IMPEncoderPack,
            stream.packCount as usize,
        );
        for pack in stream_packs {
            if pack.length > 0 {
                let jpeg_buffer = vec![];
                let mut cursor = Cursor::new(jpeg_buffer);
                let rem_size = stream.streamSize - pack.offset;
                if rem_size < pack.length {
                    let src1 = std::slice::from_raw_parts(
                        (stream.virAddr + pack.offset) as *const u8,
                        rem_size as usize,
                    );
                    cursor.write_all(src1).unwrap();
                    let src2 = std::slice::from_raw_parts(
                        stream.virAddr as *const u8,
                        (pack.length - rem_size) as usize,
                    );
                    cursor.write_all(src2).unwrap();
                } else {
                    let src = std::slice::from_raw_parts(
                        (stream.virAddr + pack.offset) as *const u8,
                        pack.length as usize,
                    );
                    cursor.write_all(src).unwrap();
                }

                tx.send(cursor.into_inner()).unwrap();
            }
        }

        if IMP_Encoder_ReleaseStream(2, &mut stream) < 0 {
            error!("IMP_Encoder_ReleaseStream failed");
            continue;
        }
    }
}
