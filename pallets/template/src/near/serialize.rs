pub mod base64_format {
	use serde::{de, Deserialize, Deserializer, Serializer};

	pub fn to_base64<T: AsRef<[u8]>>(input: T) -> String {
		base64::encode(&input)
	}

	pub fn from_base64(s: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
		base64::decode(s).map_err(|err| err.into())
	}

	pub fn serialize<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
		T: AsRef<[u8]>,
	{
		serializer.serialize_str(&to_base64(data))
	}

	pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
	where
		D: Deserializer<'de>,
		T: From<Vec<u8>>,
	{
		let s = String::deserialize(deserializer)?;
		from_base64(&s)
			.map_err(|err| de::Error::custom(err.to_string()))
			.map(Into::into)
	}
}

/// Serialises number as a string; deserialises either as a string or number.
///
/// This format works for `u64`, `u128`, `Option<u64>` and `Option<u128>` types.
/// When serialising, numbers are serialised as decimal strings.  When
/// deserialising, strings are parsed as decimal numbers while numbers are
/// interpreted as is.
pub mod dec_format {
	use serde::{de, Deserializer, Serializer};

	#[derive(thiserror::Error, Debug)]
	#[error("cannot parse from unit")]
	pub struct ParseUnitError;

	/// Abstraction between integers that we serialise.
	pub trait DecType: Sized {
		/// Formats number as a decimal string; passes `None` as is.
		fn serialize(&self) -> Option<String>;

		/// Constructs Self from a `null` value.  Returns error if this type
		/// does not accept `null` values.
		fn try_from_unit() -> Result<Self, ParseUnitError> {
			Err(ParseUnitError)
		}

		/// Tries to parse decimal string as an integer.
		fn try_from_str(value: &str) -> Result<Self, std::num::ParseIntError>;

		/// Constructs Self from a 64-bit unsigned integer.
		fn from_u64(value: u64) -> Self;
	}

	impl DecType for u64 {
		fn serialize(&self) -> Option<String> {
			Some(self.to_string())
		}
		fn try_from_str(value: &str) -> Result<Self, std::num::ParseIntError> {
			Self::from_str_radix(value, 10)
		}
		fn from_u64(value: u64) -> Self {
			value
		}
	}

	impl DecType for u128 {
		fn serialize(&self) -> Option<String> {
			Some(self.to_string())
		}
		fn try_from_str(value: &str) -> Result<Self, std::num::ParseIntError> {
			Self::from_str_radix(value, 10)
		}
		fn from_u64(value: u64) -> Self {
			value.into()
		}
	}

	impl<T: DecType> DecType for Option<T> {
		fn serialize(&self) -> Option<String> {
			self.as_ref().and_then(DecType::serialize)
		}
		fn try_from_unit() -> Result<Self, ParseUnitError> {
			Ok(None)
		}
		fn try_from_str(value: &str) -> Result<Self, std::num::ParseIntError> {
			Some(T::try_from_str(value)).transpose()
		}
		fn from_u64(value: u64) -> Self {
			Some(T::from_u64(value))
		}
	}

	struct Visitor<T>(core::marker::PhantomData<T>);

	impl<'de, T: DecType> de::Visitor<'de> for Visitor<T> {
		type Value = T;

		fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
			fmt.write_str("a non-negative integer as a string")
		}

		fn visit_unit<E: de::Error>(self) -> Result<T, E> {
			T::try_from_unit().map_err(|_| de::Error::invalid_type(de::Unexpected::Option, &self))
		}

		fn visit_u64<E: de::Error>(self, value: u64) -> Result<T, E> {
			Ok(T::from_u64(value))
		}

		fn visit_str<E: de::Error>(self, value: &str) -> Result<T, E> {
			T::try_from_str(value).map_err(de::Error::custom)
		}
	}

	pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
	where
		D: Deserializer<'de>,
		T: DecType,
	{
		deserializer.deserialize_any(Visitor(Default::default()))
	}

	pub fn serialize<S, T>(num: &T, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
		T: DecType,
	{
		match num.serialize() {
			Some(value) => serializer.serialize_str(&value),
			None => serializer.serialize_none(),
		}
	}
}
