#![allow(dead_code, unreachable_patterns, non_camel_case_types)]

mod f1ap{
    include!(concat!(env!("OUT_DIR"), "/f1ap.rs"));
}

fn main() {
    use asn1_codecs::{aper::AperCodec, PerCodecData};
    eprintln!("F1AP");

    let _ = env_logger::init();

    let f1ap_byte_str = "000600260000050028000200e80029400200e8000040010a00324009080002e6264783aad30040400120";
    let f1ap_data = hex::decode(f1ap_byte_str).unwrap();
    let mut codec_data = PerCodecData::from_slice_aper(&f1ap_data);
    let f1ap_pdu = f1ap::F1AP_PDU::aper_decode(&mut codec_data).unwrap();

    eprintln!("f1ap_pdu: {:#?}", f1ap_pdu);
    let mut encode_codec_data = PerCodecData::new_aper();
    let result = f1ap_pdu.aper_encode(&mut encode_codec_data);
    eprintln!("result: {:#?}", result);
    let f1ap_encoded_data = encode_codec_data.get_inner().unwrap();
    eprintln!("Original: {}", hex::encode(&f1ap_encoded_data));
    eprintln!("Encoded: {}", f1ap_byte_str);
    eprintln!("{}", f1ap_data.len() == f1ap_encoded_data.len());
    let f1ap_pdu = f1ap::F1AP_PDU::aper_decode(&mut encode_codec_data);
    eprintln!("f1ap_pdu: {:#?}", f1ap_pdu);

    // Error response!
    let response_data = [
        0x40, 0x06, 0x00, 0x0f, 0x00, 0x00, 0x02, 0x00,
        0x28, 0x00, 0x02, 0x00, 0xe8, 0x00, 0x29, 0x00,
        0x02, 0x00, 0xe8
    ];
    let mut codec_data = PerCodecData::from_slice_aper(&response_data);
    let f1ap_pdu = f1ap::F1AP_PDU::aper_decode(&mut codec_data);
    eprintln!("f1ap_pdu: {:?}", f1ap_pdu);
}
