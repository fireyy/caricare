use std::error::Error;
use std::fmt::{self, Display};
use std::str::FromStr;
use std::{borrow::Cow, collections::HashMap};

#[derive(Clone, Debug, Default)]
pub struct Query {
    inner: HashMap<QueryKey, QueryValue>,
}

impl Query {
    /// Creates an empty `Query`.
    ///
    /// The hash map is initially created with a capacity of 0, so it will not allocate until it
    /// is first inserted into.
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Creates an empty `Query` with at least the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    /// Inserts a key-value pair into the map.
    pub fn insert(&mut self, key: impl Into<QueryKey>, value: impl Into<QueryValue>) {
        self.inner.insert(key.into(), value.into());
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: impl Into<QueryKey>) -> Option<&QueryValue> {
        self.inner.get(&key.into())
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    pub fn remove(&mut self, key: impl Into<QueryKey>) -> Option<QueryValue> {
        self.inner.remove(&key.into())
    }

    pub fn to_hashmap(&self) -> HashMap<&str, Option<&str>> {
        let mut hashmap = HashMap::new();
        hashmap.insert("list-type", Some("2"));
        for (key, val) in self.inner.iter() {
            let key = key.as_ref();
            let val = val.as_ref();
            hashmap.insert(key, Some(val));
        }
        hashmap
    }

    /// 将查询参数拼成 aliyun 接口需要的格式
    pub fn to_oss_string(&self) -> String {
        let mut query_str = String::from("list-type=2");
        for (key, value) in self.inner.iter() {
            query_str += "&";
            query_str += key.as_ref();
            query_str += "=";
            query_str += value.as_ref();
        }
        query_str
    }

    /// 转化成 url 参数的形式
    /// a=foo&b=bar
    pub fn to_url_query(&self) -> String {
        self.inner
            .iter()
            .map(|(k, v)| {
                let mut res = String::with_capacity(k.as_ref().len() + v.as_ref().len() + 1);
                res.push_str(k.as_ref());
                res.push('=');
                res.push_str(v.as_ref());
                res
            })
            .collect::<Vec<_>>()
            .join("&")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum QueryKey {
    /// 对Object名字进行分组的字符。所有Object名字包含指定的前缀，第一次出现delimiter字符之间的Object作为一组元素（即CommonPrefixes）
    /// 示例值 `/`
    Delimiter,

    /// 设定从start-after之后按字母排序开始返回Object。
    /// start-after用来实现分页显示效果，参数的长度必须小于1024字节。
    /// 做条件查询时，即使start-after在列表中不存在，也会从符合start-after字母排序的下一个开始打印。
    StartAfter,

    /// 指定List操作需要从此token开始。您可从ListObjectsV2（GetBucketV2）结果中的NextContinuationToken获取此token。
    /// 用于分页，返回下一页的数据
    ContinuationToken,

    /// 指定返回Object的最大数。
    /// 取值：大于0小于等于1000
    MaxKeys,

    /// # 限定返回文件的Key必须以prefix作为前缀。
    /// 如果把prefix设为某个文件夹名，则列举以此prefix开头的文件，即该文件夹下递归的所有文件和子文件夹。
    ///
    /// 在设置prefix的基础上，将delimiter设置为正斜线（/）时，返回值就只列举该文件夹下的文件，文件夹下的子文件夹名返回在CommonPrefixes中，
    /// 子文件夹下递归的所有文件和文件夹不显示。
    ///
    /// 例如，一个Bucket中有三个Object，分别为fun/test.jpg、fun/movie/001.avi和fun/movie/007.avi。如果设定prefix为fun/，
    /// 则返回三个Object；如果在prefix设置为fun/的基础上，将delimiter设置为正斜线（/），则返回fun/test.jpg和fun/movie/。
    /// ## 要求
    /// - 参数的长度必须小于1024字节。
    /// - 设置prefix参数时，不能以正斜线（/）开头。如果prefix参数置空，则默认列举Bucket内的所有Object。
    /// - 使用prefix查询时，返回的Key中仍会包含prefix。
    Prefix,

    /// 对返回的内容进行编码并指定编码的类型。
    EncodingType,

    /// 指定是否在返回结果中包含owner信息。
    FetchOwner,

    /// 自定义
    Custom(Cow<'static, str>),
}

impl AsRef<str> for QueryKey {
    /// ```
    /// # use aliyun_oss_client::QueryKey;
    /// # use std::borrow::Cow;
    /// assert_eq!(QueryKey::Delimiter.as_ref(), "delimiter");
    /// assert_eq!(QueryKey::StartAfter.as_ref(), "start-after");
    /// assert_eq!(QueryKey::ContinuationToken.as_ref(), "continuation-token");
    /// assert_eq!(QueryKey::MaxKeys.as_ref(), "max-keys");
    /// assert_eq!(QueryKey::Prefix.as_ref(), "prefix");
    /// assert_eq!(QueryKey::EncodingType.as_ref(), "encoding-type");
    /// assert_eq!(QueryKey::Custom(Cow::Borrowed("abc")).as_ref(), "abc");
    /// ```
    fn as_ref(&self) -> &str {
        use QueryKey::*;

        match *self {
            Delimiter => "delimiter",
            StartAfter => "start-after",
            ContinuationToken => "continuation-token",
            MaxKeys => "max-keys",
            Prefix => "prefix",
            EncodingType => "encoding-type",
            FetchOwner => unimplemented!("parse xml not support fetch owner"),
            Custom(ref str) => str,
        }
    }
}

impl From<String> for QueryKey {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}
impl From<&'static str> for QueryKey {
    fn from(date: &'static str) -> Self {
        Self::from_static(date)
    }
}

impl FromStr for QueryKey {
    type Err = InvalidQueryKey;
    /// 示例
    /// ```
    /// # use aliyun_oss_client::types::QueryKey;
    /// let value: QueryKey = "abc".into();
    /// assert!(value == QueryKey::from_static("abc"));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.to_owned()))
    }
}

impl QueryKey {
    /// # Examples
    /// ```
    /// # use aliyun_oss_client::QueryKey;
    /// # use assert_matches::assert_matches;
    /// let key = QueryKey::new("delimiter");
    /// assert!(key == QueryKey::Delimiter);
    /// assert!(QueryKey::new("start-after") == QueryKey::StartAfter);
    /// assert!(QueryKey::new("continuation-token") == QueryKey::ContinuationToken);
    /// assert!(QueryKey::new("max-keys") == QueryKey::MaxKeys);
    /// assert!(QueryKey::new("prefix") == QueryKey::Prefix);
    /// assert!(QueryKey::new("encoding-type") == QueryKey::EncodingType);
    ///
    /// let key = QueryKey::new("abc");
    /// assert_matches!(key, QueryKey::Custom(_));
    /// ```
    /// *`fetch-owner` 功能未实现，特殊说明*
    pub fn new(val: impl Into<Cow<'static, str>>) -> Self {
        use QueryKey::*;

        let val = val.into();
        if val.contains("delimiter") {
            Delimiter
        } else if val.contains("start-after") {
            StartAfter
        } else if val.contains("continuation-token") {
            ContinuationToken
        } else if val.contains("max-keys") {
            MaxKeys
        } else if val.contains("prefix") {
            Prefix
        } else if val.contains("encoding-type") {
            EncodingType
        } else if val.contains("fetch-owner") {
            unimplemented!("parse xml not support fetch owner");
        } else {
            Custom(val)
        }
    }

    /// # Examples
    /// ```
    /// # use aliyun_oss_client::QueryKey;
    /// # use assert_matches::assert_matches;
    /// let key = QueryKey::from_static("delimiter");
    /// assert!(key == QueryKey::Delimiter);
    /// assert!(QueryKey::from_static("start-after") == QueryKey::StartAfter);
    /// assert!(QueryKey::from_static("continuation-token") == QueryKey::ContinuationToken);
    /// assert!(QueryKey::from_static("max-keys") == QueryKey::MaxKeys);
    /// assert!(QueryKey::from_static("prefix") == QueryKey::Prefix);
    /// assert!(QueryKey::from_static("encoding-type") == QueryKey::EncodingType);
    ///
    /// let key = QueryKey::from_static("abc");
    /// assert_matches!(key, QueryKey::Custom(_));
    /// ```
    /// *`fetch-owner` 功能未实现，特殊说明*
    pub fn from_static(val: &'static str) -> Self {
        use QueryKey::*;

        if val.contains("delimiter") {
            Delimiter
        } else if val.contains("start-after") {
            StartAfter
        } else if val.contains("continuation-token") {
            ContinuationToken
        } else if val.contains("max-keys") {
            MaxKeys
        } else if val.contains("prefix") {
            Prefix
        } else if val.contains("encoding-type") {
            EncodingType
        } else if val.contains("fetch-owner") {
            unimplemented!("parse xml not support fetch owner");
        } else {
            Custom(Cow::Borrowed(val))
        }
    }
}

/// 异常的查询条件键
#[derive(Debug)]
pub struct InvalidQueryKey;

impl Error for InvalidQueryKey {}

impl Display for InvalidQueryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid query key")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct QueryValue(Cow<'static, str>);

impl AsRef<str> for QueryValue {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for QueryValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for QueryValue {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}
impl From<&'static str> for QueryValue {
    fn from(date: &'static str) -> Self {
        Self::from_static(date)
    }
}

impl PartialEq<&str> for QueryValue {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        &self.0 == other
    }
}

impl From<u8> for QueryValue {
    /// 数字转 Query 值
    ///
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// let query = Query::from_iter([("max_keys", 100u8)]);
    /// let query = Query::from_iter([(QueryKey::MaxKeys, 100u8)]);
    /// ```
    fn from(num: u8) -> Self {
        Self(Cow::Owned(num.to_string()))
    }
}

impl PartialEq<u8> for QueryValue {
    #[inline]
    fn eq(&self, other: &u8) -> bool {
        self.to_string() == other.to_string()
    }
}

impl From<u16> for QueryValue {
    /// 数字转 Query 值
    ///
    /// ```
    /// use aliyun_oss_client::Query;
    /// let query = Query::from_iter([("max_keys", 100u16)]);
    /// ```
    fn from(num: u16) -> Self {
        Self(Cow::Owned(num.to_string()))
    }
}

impl PartialEq<u16> for QueryValue {
    #[inline]
    fn eq(&self, other: &u16) -> bool {
        self.to_string() == other.to_string()
    }
}

impl From<bool> for QueryValue {
    /// bool 转 Query 值
    ///
    /// ```
    /// use aliyun_oss_client::Query;
    /// let query = Query::from_iter([("abc", "false")]);
    /// ```
    fn from(b: bool) -> Self {
        if b {
            Self::from_static("true")
        } else {
            Self::from_static("false")
        }
    }
}

impl FromStr for QueryValue {
    type Err = InvalidQueryValue;
    /// 示例
    /// ```
    /// # use aliyun_oss_client::types::QueryValue;
    /// let value: QueryValue = "abc".parse().unwrap();
    /// assert!(value == "abc");
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Cow::Owned(s.to_owned())))
    }
}

/// 异常的查询值
#[derive(Debug)]
pub struct InvalidQueryValue;

impl Error for InvalidQueryValue {}

impl Display for InvalidQueryValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid query value")
    }
}

impl QueryValue {
    /// Creates a new `QueryValue` from the given string.
    pub fn new(val: impl Into<Cow<'static, str>>) -> Self {
        Self(val.into())
    }

    /// Const function that creates a new `QueryValue` from a static str.
    pub const fn from_static(val: &'static str) -> Self {
        Self(Cow::Borrowed(val))
    }
}
