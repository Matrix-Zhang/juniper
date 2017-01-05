use serde::{de, ser};
use std::collections::HashMap;

use ::{GraphQLError, Value};
use ast::InputValue;
use executor::ExecutionError;
use parser::{ParseError, Spanning, SourcePosition};
use validation::RuleError;

impl ser::Serialize for ExecutionError {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer,
    {
        let mut state = try!(serializer.serialize_map(Some(3)));

        try!(serializer.serialize_map_key(&mut state, "message"));
        try!(serializer.serialize_map_value(&mut state, self.message()));

        try!(serializer.serialize_map_key(&mut state, "locations"));
        try!(serializer.serialize_map_value(&mut state, vec![self.location()]));

        try!(serializer.serialize_map_key(&mut state, "path"));
        try!(serializer.serialize_map_value(&mut state, self.path()));

        serializer.serialize_map_end(state)
    }
}

impl<'a> ser::Serialize for GraphQLError<'a> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer,
    {
        match *self {
            GraphQLError::ParseError(ref err) => vec![err].serialize(serializer),
            GraphQLError::ValidationError(ref errs) => errs.serialize(serializer),
            GraphQLError::NoOperationProvided => {
                serializer.serialize_str("Must provide an operation")
            },
            GraphQLError::MultipleOperationsProvided => {
                serializer.serialize_str("Must provide operation name if query contains multiple operations")
            },
            GraphQLError::UnknownOperationName => {
                serializer.serialize_str("Unknown operation")
            },
        }
    }
}

impl de::Deserialize for InputValue {
    fn deserialize<D>(deserializer: &mut D) -> Result<InputValue, D::Error>
        where D: de::Deserializer,
    {
        struct InputValueVisitor;

        impl de::Visitor for InputValueVisitor {
            type Value = InputValue;

            fn visit_bool<E>(&mut self, value: bool) -> Result<InputValue, E> {
                Ok(InputValue::boolean(value))
            }

            fn visit_i64<E>(&mut self, value: i64) -> Result<InputValue, E> {
                Ok(InputValue::int(value))
            }

            fn visit_u64<E>(&mut self, value: u64) -> Result<InputValue, E>
                where E: de::Error,
            {
                self.visit_f64(value as f64)
            }

            fn visit_f64<E>(&mut self, value: f64) -> Result<InputValue, E> {
                Ok(InputValue::float(value))
            }

            fn visit_str<E>(&mut self, value: &str) -> Result<InputValue, E>
                where E: de::Error,
            {
                self.visit_string(value.into())
            }

            fn visit_string<E>(&mut self, value: String) -> Result<InputValue, E> {
                Ok(InputValue::string(value))
            }

            fn visit_none<E>(&mut self) -> Result<InputValue, E> {
                Ok(InputValue::null())
            }

            fn visit_unit<E>(&mut self) -> Result<InputValue, E> {
                Ok(InputValue::null())
            }

            fn visit_seq<V>(&mut self, visitor: V) -> Result<InputValue, V::Error>
                where V: de::SeqVisitor,
            {
                let values = try!(de::impls::VecVisitor::new().visit_seq(visitor));
                Ok(InputValue::list(values))
            }

            fn visit_map<V>(&mut self, visitor: V) -> Result<InputValue, V::Error>
                where V: de::MapVisitor,
            {
                let values = try!(de::impls::HashMapVisitor::<String, InputValue, _>::new()
                    .visit_map(visitor));
                Ok(InputValue::object(values))
            }
        }

        deserializer.deserialize(InputValueVisitor)
    }
}

impl ser::Serialize for InputValue {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer,
    {
        match *self {
            InputValue::Null | InputValue::Variable(_) => serializer.serialize_unit(),
            InputValue::Int(v) => serializer.serialize_i64(v),
            InputValue::Float(v) => serializer.serialize_f64(v),
            InputValue::String(ref v) | InputValue::Enum(ref v) => serializer.serialize_str(v),
            InputValue::Boolean(v) => serializer.serialize_bool(v),
            InputValue::List(ref v) => {
                v.iter().map(|x| x.item.clone()).collect::<Vec<_>>().serialize(serializer)
            },
            InputValue::Object(ref v) => {
                v.iter()
                    .map(|&(ref k, ref v)| (k.item.clone(), v.item.clone()))
                    .collect::<HashMap<_, _>>()
                    .serialize(serializer)
            },
        }
    }
}

impl ser::Serialize for RuleError {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer,
    {
        let mut state = try!(serializer.serialize_map(Some(2)));

        try!(serializer.serialize_map_key(&mut state, "message"));
        try!(serializer.serialize_map_value(&mut state, self.message()));

        try!(serializer.serialize_map_key(&mut state, "locations"));
        try!(serializer.serialize_map_value(&mut state, self.locations()));

        serializer.serialize_map_end(state)
    }
}

impl ser::Serialize for SourcePosition {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer,
    {
        let mut state = try!(serializer.serialize_map(Some(2)));

        try!(serializer.serialize_map_key(&mut state, "line"));
        try!(serializer.serialize_map_value(&mut state, self.line() + 1));

        try!(serializer.serialize_map_key(&mut state, "column"));
        try!(serializer.serialize_map_value(&mut state, self.column() + 1));

        serializer.serialize_map_end(state)
    }
}

impl<'a> ser::Serialize for Spanning<ParseError<'a>> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer,
    {
        let mut state = try!(serializer.serialize_map(Some(2)));

        let message = format!("{}", self.item);
        try!(serializer.serialize_map_key(&mut state, "message"));
        try!(serializer.serialize_map_value(&mut state, message));

        let mut location = HashMap::new();
        location.insert("line".to_owned(), self.start.line() + 1);
        location.insert("column".to_owned(), self.start.column() + 1);

        let locations = vec![location];

        try!(serializer.serialize_map_key(&mut state, "locations"));
        try!(serializer.serialize_map_value(&mut state, locations));

        serializer.serialize_map_end(state)
    }
}

impl ser::Serialize for Value {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer,
    {
        match *self {
            Value::Null => serializer.serialize_unit(),
            Value::Int(v) => serializer.serialize_i64(v),
            Value::Float(v) => serializer.serialize_f64(v),
            Value::String(ref v) => serializer.serialize_str(v),
            Value::Boolean(v) => serializer.serialize_bool(v),
            Value::List(ref v) => v.serialize(serializer),
            Value::Object(ref v) => v.serialize(serializer),
        }
    }
}
