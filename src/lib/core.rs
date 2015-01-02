// TODO: drop all panic!

use std::mem;
use std::raw;
use std::fmt;
use std::default::Default;
use std::intrinsics::TypeId;

use clear::Clear;
use reflect::MessageDescriptor;
use reflect::EnumDescriptor;
use reflect::EnumValueDescriptor;
use unknown::UnknownFields;
use stream::WithCodedInputStream;
use stream::WithCodedOutputStream;
use stream::CodedInputStream;
use stream::CodedOutputStream;
use stream::with_coded_output_stream_to_bytes;
use error::ProtobufResult;


pub trait Message : PartialEq + fmt::Show + Clear {
    // also all generated Message types implement Clone, Default traits

    fn new() -> Self;
    // all required fields set
    fn is_initialized(&self) -> bool;
    fn merge_from(&mut self, is: &mut CodedInputStream) -> ProtobufResult<()>;

    // sizes of this messages (and nested messages) must be cached
    // by calling `compute_size` prior to this call
    fn write_to_with_cached_sizes(&self, os: &mut CodedOutputStream) -> ProtobufResult<()>;

    // compute and cache size of this message and all nested messages
    fn compute_size(&self) -> u32;

    // get size previously computed by `compute_size`
    fn get_cached_size(&self) -> u32;

    fn write_to(&self, os: &mut CodedOutputStream) -> ProtobufResult<()> {
        self.check_initialized();

        // cache sizes
        self.compute_size();
        try!(self.write_to_with_cached_sizes(os));

        // TODO: assert we've written same number of bytes as computed

        Ok(())
    }

    fn write_length_delimited_to(&self, os: &mut CodedOutputStream) -> ProtobufResult<()> {
        let size = self.compute_size();
        try!(os.write_raw_varint32(size));
        try!(self.write_to_with_cached_sizes(os));

        // TODO: assert we've written same number of bytes as computed

        Ok(())
    }

    fn merge_from_bytes(&mut self, bytes: &[u8]) -> ProtobufResult<()> {
        let mut is = CodedInputStream::from_bytes(bytes);
        self.merge_from(&mut is)
    }

    fn check_initialized(&self) {
        // TODO: report which fields are not initialized
        assert!(self.is_initialized());
    }

    fn write_to_writer(&self, w: &mut Writer) -> ProtobufResult<()> {
        w.with_coded_output_stream(|os| {
            self.write_to(os)
        })
    }

    fn write_to_vec(&self, v: &mut Vec<u8>) -> ProtobufResult<()> {
        v.with_coded_output_stream(|os| {
            self.write_to(os)
        })
    }

    fn write_to_bytes(&self) -> ProtobufResult<Vec<u8>> {
        // TODO: compute message size and reserve that size
        let mut v = Vec::new();
        try!(self.write_to_vec(&mut v));
        Ok(v)
    }

    fn write_length_delimited_to_writer(&self, w: &mut Writer) -> ProtobufResult<()> {
        w.with_coded_output_stream(|os| {
            self.write_length_delimited_to(os)
        })
    }

    fn write_length_delimited_to_bytes(&self) -> ProtobufResult<Vec<u8>> {
        with_coded_output_stream_to_bytes(|os| {
            self.write_length_delimited_to(os)
        })
    }

    fn get_unknown_fields<'s>(&'s self) -> &'s UnknownFields;
    fn mut_unknown_fields<'s>(&'s mut self) -> &'s mut UnknownFields;

    fn descriptor(&self) -> &'static MessageDescriptor {
        Message::descriptor_static(None::<Self>)
    }

    // http://stackoverflow.com/q/20342436/15018
    fn descriptor_static(_: Option<Self>) -> &'static MessageDescriptor {
        panic!("descriptor_static is not implemented for message, \
            LITE_RUNTIME must be used");
    }

    fn type_id(&self) -> TypeId {
        panic!();
    }

    // Rust does not allow implementation of trait for trait:
    // impl<M : Message> fmt::Show for M {
    // ...
    // }
    fn fmt_impl(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::text_format::fmt(self, f)
    }
}

pub fn message_is<M : 'static + Message>(m: &Message) -> bool {
    TypeId::of::<M>() == m.type_id()
}

pub fn message_down_cast<'a, M : 'static + Message>(m: &'a Message) -> &'a M {
    assert!(message_is::<M>(m));
    unsafe {
        // TODO: really weird
        let r: raw::TraitObject = mem::transmute(m);
        mem::transmute(r.data)
    }
}


pub trait ProtobufEnum : Eq {
    fn value(&self) -> i32;

    fn from_i32(v: i32) -> Option<Self>;

    fn descriptor(&self) -> &'static EnumValueDescriptor {
        self.enum_descriptor().value_by_number(self.value())
    }

    fn enum_descriptor(&self) -> &'static EnumDescriptor {
        ProtobufEnum::enum_descriptor_static(None::<Self>)
    }

    // http://stackoverflow.com/q/20342436/15018
    fn enum_descriptor_static(_: Option<Self>) -> &'static EnumDescriptor {
        panic!();
    }
}

pub fn parse_from<M : Message>(is: &mut CodedInputStream) -> ProtobufResult<M> {
    let mut r: M = Message::new();
    try!(r.merge_from(is));
    r.check_initialized();
    Ok(r)
}

pub fn parse_from_reader<M : Message>(reader: &mut Reader) -> ProtobufResult<M> {
    reader.with_coded_input_stream(|is| {
        parse_from::<M>(is)
    })
}

pub fn parse_from_bytes<M : Message>(bytes: &[u8]) -> ProtobufResult<M> {
    bytes.with_coded_input_stream(|is| {
        parse_from::<M>(is)
    })
}

pub fn parse_length_delimited_from<M : Message>(is: &mut CodedInputStream) -> ProtobufResult<M> {
    is.read_message::<M>()
}

pub fn parse_length_delimited_from_reader<M : Message>(r: &mut Reader) -> ProtobufResult<M> {
    // TODO: wrong: we may read length first, and then read exact number of bytes needed
    r.with_coded_input_stream(|is| {
        is.read_message::<M>()
    })
}

pub fn parse_length_delimited_from_bytes<M : Message>(bytes: &[u8]) -> ProtobufResult<M> {
    bytes.with_coded_input_stream(|is| {
        is.read_message::<M>()
    })
}


