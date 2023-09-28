use std::{borrow::Cow, collections::HashMap};

use chrono::NaiveDate;
use serde::{
    ser::SerializeMap, Deserialize, Serialize, Serializer, __private::ser::FlatMapSerializer,
};
use serde_json::Value;

use crate::common::{Id, UnsignedInt, UtcDate};

#[derive(Deserialize, Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct TypeWrapper<T>(T);

impl<T: Serialize + TypedStruct> Serialize for TypeWrapper<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("@type", T::KIND)?;
        self.0.serialize(FlatMapSerializer(&mut map))?;
        map.end()
    }
}

pub trait TypedStruct {
    const KIND: &'static str;
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", tag = "@type")]
pub enum Data<'a> {
    Card(#[serde(borrow)] Card<'a>),
    CardGroup(CardGroup<'a>),
}

/// A CardGroup object represents a group of cards. Its members may be Cards or CardGroups.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CardGroup<'a> {
    /// An identifier, used to associate the object as the same across different
    /// systems, addressbooks and views.
    #[serde(borrow)]
    uid: Id<'a>,
    /// The set is represented as an object, with each key being the uid of another Card or
    /// CardGroup. The value for each key in the object MUST be true.
    members: HashMap<Id<'a>, bool>,
    /// The user-visible name for the group, e.g. "Friends". This may be any UTF-8 string of at
    /// least 1 character in length and maximum 255 octets in size. The same name may be used by
    /// two different groups.
    #[serde(default)]
    name: Cow<'a, str>,
    /// The card that represents this group.
    card: Option<Card<'a>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Card<'a> {
    /// An identifier, used to associate the object as the same across different
    /// systems, addressbooks and views.
    #[serde(borrow)]
    uid: Id<'a>,
    /// The identifier for the product that created the Card object.
    prod_id: Option<Cow<'a, str>>,
    /// The date and time when this Card object was created.
    created: Option<UtcDate>,
    /// The date and time when the data in this Card object was last modified.
    updated: Option<UtcDate>,
    /// The kind of the entity the Card represents.
    kind: Option<CardKind>,
    /// Relates the object to other Card and CardGroup objects. This is
    /// represented as a map, where each key is the
    #[serde(default)]
    related_to: HashMap<Id<'a>, TypeWrapper<Relation>>,
    /// Language used for free-form text on this card.
    language: Option<Cow<'a, str>>,
    /// The name components of the name of the entity represented by this Card.
    #[serde(default)]
    name: Vec<TypeWrapper<NameComponent<'a>>>,
    /// The full name (e.g. the personal name and surname of an individual, the
    /// name of an organization) of the entity represented by this card. The
    /// purpose of this property is to define a name, even if the individual
    /// name components are not known. In addition, it is meant to provide
    /// alternative versions of the name for internationalisation.
    ///
    /// Implementations SHOULD prefer using the name property over this one
    /// and SHOULD NOT store the concatenated name component values in this
    /// property.
    #[serde(default)]
    full_name: Cow<'a, str>,
    /// The nick names of the entity represented by this card.
    #[serde(default)]
    nick_names: Vec<Cow<'a, str>>,
    /// The companies or organization names and units associated with this
    /// card.
    #[serde(default)]
    organizations: HashMap<Id<'a>, TypeWrapper<Organization<'a>>>,
    /// The job titles or functional positions of the entity represented by
    /// this card.
    #[serde(default)]
    titles: HashMap<Id<'a>, TypeWrapper<Title<'a>>>,
    /// The email addresses to contact the entity represented by this card.
    #[serde(default)]
    emails: HashMap<Id<'a>, TypeWrapper<EmailAddress<'a>>>,
    /// The phone numbers to contact the entity represented by this card.
    #[serde(default)]
    phones: HashMap<Id<'a>, TypeWrapper<Phone<'a>>>,
    /// The online resources and services that are associated with the entity
    /// represented by this card.
    #[serde(default)]
    online: HashMap<Id<'a>, TypeWrapper<Resource<'a>>>,
    /// A map of photo ids to File objects that contain photographs or images
    /// associated with this card. A typical use case is to include an avatar for display along the
    /// contact name.
    #[serde(default)]
    photos: HashMap<Id<'a>, TypeWrapper<File<'a>>>,
    /// Defines the preferred method to contact the holder of this card.
    preferred_contact_method: Option<PreferredContactMethod>,
    /// Defines the preferred languages for contacting the entity associated with this card. The
    /// keys in the object MUST be [RFC5646] language tags. The values are a (possibly empty) list
    /// of contact language preferences for this language. A valid ContactLanguage object MUST have
    /// at least one of its properties set.
    #[serde(default)]
    preferred_contact_languages: HashMap<String, TypeWrapper<ContactLanguage>>,
    /// A map of address ids to Address objects, containing physical locations.
    #[serde(default)]
    address: HashMap<Id<'a>, TypeWrapper<Address<'a>>>,
    /// A map of language tags [RFC5646] to patches, which localize a property value into the
    /// locale of the respective language tag.
    ///
    /// A patch MUST NOT target the localizations property.
    #[serde(default)]
    localizations: HashMap<Cow<'a, str>, Value>,
    /// These are memorable dates and events for the entity represented by this card.
    #[serde(default)]
    anniversaries: HashMap<Id<'a>, TypeWrapper<Anniversary<'a>>>,
    /// Defines personal information about the entity represented by this card.
    #[serde(default)]
    personal_info: HashMap<Id<'a>, TypeWrapper<PersonalInfo<'a>>>,
    /// Arbitrary notes about the entity represented by this card.
    #[serde(default)]
    notes: Cow<'a, str>,
    /// The set of free-text or URI categories that relate to the card. The set is represented as
    /// an object, with each key being a category. The value for each key in the object MUST be
    /// true.
    #[serde(default)]
    categories: HashMap<Cow<'a, str>, bool>,
    ///  Maps identifiers of custom time zones to their time zone definitions. For a description of
    /// this property see the timeZones property definition in [RFC8984].
    #[serde(default)]
    time_zones: HashMap<Cow<'a, str>, Value>,
}

/// Defines personal information about the entity represented by this card.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersonalInfo<'a> {
    /// Specifies the type for this personal information.
    #[serde(rename = "type")]
    type_: PersonalInfoType,
    /// The actual information. This generally is free-text, but future
    /// specifications MAY restrict allowed values depending on the type of
    /// this PersonalInformation.
    value: Cow<'a, str>,
    /// Indicates the level of expertise, or engagement in hobby or interest.
    level: Option<PersonalInfoLevel>,
}

impl TypedStruct for PersonalInfo<'_> {
    const KIND: &'static str = "PersonalInfo";
}

/// Specifies the type for this personal information.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PersonalInfoType {
    /// A field of expertise or credential
    Expertise,
    /// A hobby
    Hobby,
    /// An interest
    Interest,
    /// An information not covered by the above categories
    Other,
}

/// Indicates the level of expertise, or engagement in hobby or interest.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PersonalInfoLevel {
    High,
    Medium,
    Low,
}

/// These are memorable dates and events for the entity represented by this card.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Anniversary<'a> {
    /// Specifies the type of the anniversary.
    #[serde(rename = "type")]
    type_: AnniversaryType,
    /// A label describing the value in more detail, especially if the type
    /// property has value other (but MAY be included with any type).
    #[serde(default)]
    label: Cow<'a, str>,
    /// The date of this anniversary, in the form "YYYY-MM-DD"
    /// (any part may be all 0s for unknown) or a [RFC3339] timestamp.
    date: NaiveDate,
    /// An address associated with this anniversary, e.g. the place of birth or
    /// death.
    place: Option<Address<'a>>,
}

impl TypedStruct for Anniversary<'_> {
    const KIND: &'static str = "Anniversary";
}

/// Specifies the type of the anniversary.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AnniversaryType {
    /// A birth day anniversary
    Birth,
    /// A death day anniversary
    Death,
    /// An anniversary not covered by any of the known types.
    Other,
}

/// A physical location.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Address<'a> {
    ///  The complete address, excluding type and label. This property is mainly useful to
    /// represent addresses of which the individual address components are unknown, or to provide
    /// localized representations.
    #[serde(default)]
    full_address: Cow<'a, str>,
    ///  The street address. The concatenation of the component values, separated by whitespace,
    /// SHOULD result in a valid street address for the address locale. Doing so, implementations
    /// MAY ignore any separator components. The StreetComponent object type is defined in the
    /// paragraph below.
    #[serde(default)]
    street: Vec<TypeWrapper<StreetComponent<'a>>>,
    /// The city, town, village, post town, or other locality within which the street address may
    /// be found.
    #[serde(default)]
    locality: Cow<'a, str>,
    /// The province, such as a state, county, or canton within which the locality may be found.
    #[serde(default)]
    region: Cow<'a, str>,
    /// The country name.
    #[serde(default)]
    country: Cow<'a, str>,
    /// The postal code, post code, ZIP code or other short code associated with the address by the
    /// relevant country's postal system.
    #[serde(default)]
    postcode: Cow<'a, str>,
    /// The ISO-3166-1 country code.
    #[serde(default)]
    country_code: Cow<'a, str>,
    /// A [RFC5870] "geo:" URI for the address.
    #[serde(default)]
    coordinates: Cow<'a, str>,
    /// Identifies the time zone this address is located in. This either MUST be a time zone name
    /// registered in the IANA Time Zone Database, or it MUST be a valid TimeZoneId as defined in
    /// [RFC8984]. For the latter, a corresponding time zone MUST be defined in the timeZones
    /// property.
    #[serde(default)]
    time_zone: Cow<'a, str>,
    /// The contexts of the address information.
    #[serde(default)]
    context: HashMap<AddressContext, bool>,
    /// A label describing the value in more detail.
    #[serde(default)]
    label: Cow<'a, str>,
    ///  The preference of this address in relation to other addresses.
    pref: Option<Preference>,
}

impl TypedStruct for Address<'_> {
    const KIND: &'static str = "Address";
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase", untagged)]
pub enum AddressContext {
    /// An address to be used for billing.
    Billing,
    /// An address to be used for delivering physical items
    Postal,
    /// A normal context
    Other(Context),
}

///  The street address. The concatenation of the component values, separated by whitespace, SHOULD
/// result in a valid street address for the address locale. Doing so, implementations MAY ignore
/// any separator components. The StreetComponent object type is defined in the paragraph below.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StreetComponent<'a> {
    /// The type of this street component.
    #[serde(rename = "type")]
    type_: StreetComponentKind,
    /// The value of this street component.
    value: Cow<'a, str>,
}

impl TypedStruct for StreetComponent<'_> {
    const KIND: &'static str = "StreetComponent";
}

/// The type of this street component.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StreetComponentKind {
    Name,
    Number,
    Apartment,
    Room,
    Extension,
    Direction,
    Building,
    Floor,
    PostOfficeBox,
    Separator,
    Unknown,
}

/// Defines the preferred method to contact the holder of this card.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContactLanguage {
    /// Defines the context in which to use this language.
    context: Option<Context>,
    /// Defines the preference of this language in relation to other
    /// languages of the same context.
    pref: Option<Preference>,
}

impl TypedStruct for ContactLanguage {
    const KIND: &'static str = "ContactLanguage";
}

/// Defines the preferred method to contact the holder of this card.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PreferredContactMethod {
    Emails,
    Phones,
    Online,
}

/// A map of photo ids to File objects that contain photographs or images
/// associated with this card. A typical use case is to include an avatar for display along the
/// contact name.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct File<'a> {
    /// A URI where to fetch the data of this file.
    href: Cow<'a, str>,
    /// The content-type of the file, if known.
    media_type: Cow<'a, str>,
    /// The size, in octets, of the file when fully decoded (i.e., the number
    /// of octets in the file the user would download), if known.
    size: Option<UnsignedInt>,
    /// The preference of this photo in relation to other photos.
    pref: Option<Preference>,
}

impl TypedStruct for File<'_> {
    const KIND: &'static str = "File";
}

/// The online resources and services that are associated with the entity
/// represented by this card.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Resource<'a> {
    /// resource value, where the allowed value form is defined by the the type
    /// property. In any case the value MUST NOT be empty.
    resource: Cow<'a, str>,
    /// The type of the resource value.
    #[serde(rename = "type")]
    type_: ResourceType,
    /// Used for URI resource values. Provides the media type [RFC2046] of the
    /// resource identified by the URI.
    #[serde(default)]
    media_type: Cow<'a, str>,
    /// The contexts in which to use this resource.
    #[serde(default)]
    context: HashMap<Context, bool>,
    /// A label describing the value in more detail, especially if the type
    /// property has value other (but MAY be included with any type).
    #[serde(default)]
    label: Cow<'a, str>,
    /// The preference of this resource in relation to other resources.
    pref: Option<Preference>,
}

impl TypedStruct for Resource<'_> {
    const KIND: &'static str = "Resource";
}

/// The type of the resource value.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ResourceType {
    /// The resource value is a URI, e.g. a website link. This MUST be a valid URI as defined in
    /// Section 3 of [RFC3986] and updates.
    Uri,
    /// The resource value is a username associated with the entity represented by this card (e.g.
    /// for social media, or an IM client). The label property SHOULD be included to identify what
    /// service this is for. For compatibility between clients, this label SHOULD be the canonical
    /// service name, including capitalisation. e.g. Twitter, Facebook, Skype, GitHub, XMPP. The
    /// resource value may be any non-empty free text.
    Username,
    /// The resource value is something else not covered by the above categories. A label property
    /// MAY be included to display next to the number to help the user identify its purpose. The
    /// resource value may be any non-empty free text.
    Other,
}

/// The phone numbers to contact the entity represented by this card.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Phone<'a> {
    /// The phone value, as either a URI or a free-text phone number. Typical
    /// URI schemes are the [RFC3966] tel or [RFC3261] sip schemes, but any
    /// URI scheme is allowed.
    phone: Cow<'a, str>,
    ///  The set of contact features that this phone number may be used for. The
    /// set is represented as an object, with each key being a method type. The
    /// value for each key in the object MUST be true.
    #[serde(default)]
    features: HashMap<PhoneFeature, bool>,
    // The contexts in which to use this number. The value for each
    /// key in the object MUST be true.
    #[serde(default)]
    contexts: HashMap<Context, bool>,
    /// A label describing the value in more detail, especially if the type
    /// property has value other (but MAY be included with any type).
    #[serde(default)]
    label: Cow<'a, str>,
    /// The preference of this email address in relation to other email addresses.
    pref: Option<Preference>,
}

impl TypedStruct for Phone<'_> {
    const KIND: &'static str = "Phone";
}

/// The email addresses to contact the entity represented by this card.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum PhoneFeature {
    /// The number is for calling by voice.
    Voice,
    /// The number is for sending faxes.
    Fax,
    /// The number is for a pager or beeper.
    Pager,
    /// The number supports text messages (SMS).
    Text,
    /// The number is for a cell phone.
    Cell,
    /// The number is for a device for people with hearing or speech difficulties.
    Textphone,
    /// The number supports video conferencing.
    Video,
    /// The number is for some other purpose. The label property MAY be included
    /// to display next to the number to help the user identify its purpose.
    Other,
}

/// The email addresses to contact the entity represented by this card.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct EmailAddress<'a> {
    /// The email address. This MUST be an addr-spec value as defined in
    /// Section 3.4.1 of [RFC5322].
    email: Cow<'a, str>,
    /// The contexts in which to use this email address. The value for each
    /// key in the object MUST be true.
    #[serde(default)]
    contexts: HashMap<Context, bool>,
    /// The preference of this email address in relation to other email addresses.
    pref: Option<Preference>,
}

impl TypedStruct for EmailAddress<'_> {
    const KIND: &'static str = "EmailAddress";
}

/// This data type allows to define a preference order on same-typed contact
/// information. For example, a card holder may have two email addresses and
/// prefer to be contacted with one of them.
///
/// A preference value MUST be an integer number in the range 1 and 100. Lower
/// values correspond to a higher level of preference, with 1 being most
/// preferred. If no preference is set, then the contact information MUST be
/// interpreted as being least preferred.
///
/// Note that the preference only is defined in relation to contact information
/// of the same type. For example, the preference orders within emails and
/// phone numbers are indendepent of each other. Also note that the
/// preferredContactMethod property allows to define a preferred contact method
/// across method types.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preference(u8);

/// The companies or organization names and units associated with this card.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Title<'a> {
    /// The name of this organization.
    #[serde(borrow)]
    name: Cow<'a, str>,
    /// The id of the organization in which this title is held.
    #[serde(default)]
    organization: Vec<Id<'a>>,
}

impl TypedStruct for Title<'_> {
    const KIND: &'static str = "Title";
}

/// The companies or organization names and units associated with this card.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Organization<'a> {
    ///  The name of this organization.
    name: Cow<'a, str>,
    ///  Additional levels of organizational unit names.
    #[serde(default)]
    units: Vec<Cow<'a, str>>,
}

impl TypedStruct for Organization<'_> {
    const KIND: &'static str = "Organization";
}

/// The name components of the name of the entity represented by this Card. Name
/// components SHOULD be ordered such that their values joined by whitespace
/// produce a valid full name of this entity. Doing so, implementations MAY
/// ignore any separator components.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NameComponent<'a> {
    value: Cow<'a, str>,
    #[serde(rename = "type")]
    type_: NameComponentKind,
}

impl TypedStruct for NameComponent<'_> {
    const KIND: &'static str = "NameComponent";
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum NameComponentKind {
    /// The value is a honorific title(s), e.g. "Mr", "Ms", "Dr".
    Prefix,
    /// The value is a personal name(s), also known as "first name", "given name".
    Personal,
    /// The value is a surname, also known as "last name", "family name".
    Surname,
    /// The value is an additional name, also known as "middle name".
    Additional,
    /// The value is a honorific suffix, e.g. "B.A.", "Esq.".
    Suffix,
    /// A separator for two name components. The value property of the component
    /// includes the verbatim separator, for example a newline character.
    Separator,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Relation {
    relation: HashMap<RelationKind, bool>,
}

impl TypedStruct for Relation {
    const KIND: &'static str = "Relation";
}

/// Contact information typically is associated with a context in which it
/// should be used. For example, someone might have distinct phone numbers
/// for work and private contexts. The Context data type enumerates common
/// contexts.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Context {
    /// The contact information may be used to contact the card holder in a
    /// private context.
    Private,
    /// The contact information may be used to contact the card holder in a
    /// professional context.
    Work,
    /// The contact information may be used to contact the card holder in some
    /// other context.
    Other,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RelationKind {
    Contact,
    Acquaintance,
    Friend,
    Met,
    CoWorker,
    Colleague,
    CoResident,
    Neighbor,
    Child,
    Parent,
    Sibling,
    Spouse,
    Kin,
    Muse,
    Crush,
    Date,
    Sweetheart,
    Me,
    Agent,
    Emergency,
}

/// The kind of the entity the Card represents.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CardKind {
    /// A single person
    Individual,
    /// An organization
    Org,
    /// A named location
    Location,
    /// A device, such as appliances, computers, or network elements
    Device,
    /// A software application
    Application,
}
