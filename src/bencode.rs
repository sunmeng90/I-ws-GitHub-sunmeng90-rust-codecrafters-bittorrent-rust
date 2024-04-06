pub mod decode;
// pub(crate) mod bencode; means that the submodule bencode is public within its crate.
// Other modules within the same crate can access and use the bencode module, but it won't be
// visible outside of the crate itself.
pub(crate) mod bencode;
mod serde;
