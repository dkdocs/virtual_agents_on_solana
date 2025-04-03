// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Instructions {
    #[prost(message, repeated, tag="1")]
    pub instructions: ::prost::alloc::vec::Vec<Instruction>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Instruction {
    #[prost(string, tag="1")]
    pub program_id: ::prost::alloc::string::String,
    #[prost(string, repeated, tag="2")]
    pub accounts: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, tag="3")]
    pub data: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transactions {
    #[prost(message, repeated, tag="1")]
    pub transactions: ::prost::alloc::vec::Vec<Transaction>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    #[prost(string, tag="1")]
    pub symbol: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
    #[prost(uint64, tag="3")]
    pub created_at: u64,
    #[prost(string, tag="4")]
    pub dev_wallet: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub total_supply: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub tx_hash: ::prost::alloc::string::String,
    #[prost(uint64, tag="7")]
    pub created_block_number: u64,
    #[prost(string, tag="8")]
    pub status: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub id: ::prost::alloc::string::String,
}
// @@protoc_insertion_point(module)
