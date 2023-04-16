#![allow(dead_code, unreachable_patterns, non_camel_case_types)]

mod rrc{
    include!(concat!(env!("OUT_DIR"), "/rrc.rs"));
}

fn main() {
    use asn1_codecs::{aper::AperCodec, PerCodecData};
    eprintln!("RRC");

    let _ = env_logger::init();

    let rrc_byte_str = "0002e6264783aad3";
    let rrc_data = hex::decode(rrc_byte_str).unwrap();
    let mut codec_data = PerCodecData::from_slice_aper(&rrc_data);
    let rrc_pdu = rrc::RRC_PDU::aper_decode(&mut codec_data).unwrap();

    eprintln!("rrc_pdu: {:#?}", rrc_pdu);
    /*
    let mut encode_codec_data = PerCodecData::new_aper();
    let result = rrc_pdu.aper_encode(&mut encode_codec_data);
    eprintln!("result: {:#?}", result);
    let rrc_encoded_data = encode_codec_data.get_inner().unwrap();
    eprintln!("Original: {}", hex::encode(&rrc_encoded_data));
    eprintln!("Encoded: {}", rrc_byte_str);
    eprintln!("{}", rrc_data.len() == rrc_encoded_data.len());
    let rrc_pdu = rrc::RRC_PDU::aper_decode(&mut encode_codec_data);
    eprintln!("rrc_pdu: {:#?}", rrc_pdu);

    // Error response!
    let response_data = [
        0x40, 0x06, 0x00, 0x0f, 0x00, 0x00, 0x02, 0x00,
        0x28, 0x00, 0x02, 0x00, 0xe8, 0x00, 0x29, 0x00,
        0x02, 0x00, 0xe8
    ];
    let mut codec_data = PerCodecData::from_slice_aper(&response_data);
    let rrc_pdu = rrc::RRC_PDU::aper_decode(&mut codec_data);
    eprintln!("rrc_pdu: {:?}", rrc_pdu);
    */
}
