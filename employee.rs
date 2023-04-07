#[derive(asn1_codecs_derive :: AperCodec, Debug)]
#[asn(type = "ENUMERATED", extensible = true, lb = "0", ub = "3")]
pub enum WorkRole {
    CASHIER,
    MANAGER,
    CHEF,
    WAITER,
}
