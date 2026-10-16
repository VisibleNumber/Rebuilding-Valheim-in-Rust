use super::*;
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Text(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
    Bytes(Vec<u8>),
}
impl Value {
    pub fn get(&self, k: &str) -> Option<&Value> {
        if let Self::Object(v) = self {
            v.iter().find(|(n, _)| n == k).map(|(_, v)| v)
        } else {
            None
        }
    }
    pub fn int(&self) -> i64 {
        if let Self::Int(v) = self {
            *v
        } else {
            panic!("expected integer")
        }
    }
    pub fn float(&self) -> f32 {
        match self {
            Self::Float(v) => *v as f32,
            Self::Int(v) => *v as f32,
            _ => panic!("expected float"),
        }
    }
    pub fn text(&self) -> &str {
        if let Self::Text(v) = self {
            v
        } else {
            panic!("expected string")
        }
    }
    pub fn array(&self) -> &[Value] {
        if let Self::Array(v) = self {
            v
        } else {
            panic!("expected array")
        }
    }
    pub fn bytes(&self) -> &[u8] {
        if let Self::Bytes(v) = self {
            v
        } else {
            panic!("expected bytes")
        }
    }
    pub fn json(&self) -> String {
        match self {
            Self::Int(v) => v.to_string(),
            Self::Float(v) => {
                if v.is_finite() {
                    v.to_string()
                } else {
                    "null".into()
                }
            }
            Self::Text(s) => quote(s),
            Self::Bytes(v) => format!("{{\"binary_size\":{}}}", v.len()),
            Self::Array(v) => format!(
                "[{}]",
                v.iter().map(Self::json).collect::<Vec<_>>().join(",")
            ),
            Self::Object(v) => format!(
                "{{{}}}",
                v.iter()
                    .map(|(k, v)| format!("{}:{}", quote(k), v.json()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}
fn quote(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '\"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if c < ' ' => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}
// Unity's common type-tree string table is a fixed binary-format dictionary.
fn common(x: u32) -> R<String> {
    let s = match x {
        0 => "AABB",
        5 => "AnimationClip",
        19 => "AnimationCurve",
        34 => "AnimationState",
        49 => "Array",
        55 => "Base",
        60 => "BitField",
        69 => "bitset",
        76 => "bool",
        81 => "char",
        86 => "ColorRGBA",
        96 => "Component",
        106 => "data",
        111 => "deque",
        117 => "double",
        124 => "dynamic_array",
        138 => "FastPropertyName",
        155 => "first",
        161 => "float",
        167 => "Font",
        172 => "GameObject",
        183 => "Generic Mono",
        196 => "GradientNEW",
        208 => "GUID",
        213 => "GUIStyle",
        222 => "int",
        226 => "list",
        231 => "long long",
        241 => "map",
        245 => "Matrix4x4f",
        256 => "MdFour",
        263 => "MonoBehaviour",
        277 => "MonoScript",
        288 => "m_ByteSize",
        299 => "m_Curve",
        307 => "m_EditorClassIdentifier",
        331 => "m_EditorHideFlags",
        349 => "m_Enabled",
        359 => "m_ExtensionPtr",
        374 => "m_GameObject",
        387 => "m_Index",
        395 => "m_IsArray",
        405 => "m_IsStatic",
        416 => "m_MetaFlag",
        427 => "m_Name",
        434 => "m_ObjectHideFlags",
        452 => "m_PrefabInternal",
        469 => "m_PrefabParentObject",
        490 => "m_Script",
        499 => "m_StaticEditorFlags",
        519 => "m_Type",
        526 => "m_Version",
        536 => "Object",
        543 => "pair",
        548 => "PPtr<Component>",
        564 => "PPtr<GameObject>",
        581 => "PPtr<Material>",
        596 => "PPtr<MonoBehaviour>",
        616 => "PPtr<MonoScript>",
        633 => "PPtr<Object>",
        646 => "PPtr<Prefab>",
        659 => "PPtr<Sprite>",
        672 => "PPtr<TextAsset>",
        688 => "PPtr<Texture>",
        702 => "PPtr<Texture2D>",
        718 => "PPtr<Transform>",
        734 => "Prefab",
        741 => "Quaternionf",
        753 => "Rectf",
        759 => "RectInt",
        767 => "RectOffset",
        778 => "second",
        785 => "set",
        789 => "short",
        795 => "size",
        800 => "SInt16",
        807 => "SInt32",
        814 => "SInt64",
        821 => "SInt8",
        827 => "staticvector",
        840 => "string",
        847 => "TextAsset",
        857 => "TextMesh",
        866 => "Texture",
        874 => "Texture2D",
        884 => "Transform",
        894 => "TypelessData",
        907 => "UInt16",
        914 => "UInt32",
        921 => "UInt64",
        928 => "UInt8",
        934 => "unsigned int",
        947 => "unsigned long long",
        966 => "unsigned short",
        981 => "vector",
        988 => "Vector2f",
        997 => "Vector3f",
        1006 => "Vector4f",
        1015 => "m_ScriptingClassIdentifier",
        1042 => "Gradient",
        1051 => "Type*",
        1057 => "int2_storage",
        1070 => "int3_storage",
        1083 => "BoundsInt",
        1093 => "m_CorrespondingSourceObject",
        1121 => "m_PrefabInstance",
        1138 => "m_PrefabAsset",
        1152 => "FileSize",
        1161 => "Hash128",
        _ => return Err(bad(&format!("unknown common string {x}"))),
    };
    Ok(s.into())
}
struct Node {
    level: u8,
    ty: String,
    name: String,
    size: i32,
    flags: u32,
    end: usize,
}
fn table_string(st: &[u8], v: u32) -> R<String> {
    if v & 0x80000000 != 0 {
        return common(v & 0x7fffffff);
    }
    let s = st.get(v as usize..).ok_or_else(|| bad("string offset"))?;
    let n = s
        .iter()
        .position(|x| *x == 0)
        .ok_or_else(|| bad("string terminator"))?;
    Ok(String::from_utf8(s[..n].to_vec())?)
}
pub fn read(tree: &[u8], st: &[u8], data: &[u8]) -> R<Value> {
    if tree.is_empty() || tree.len() % 32 != 0 {
        return Err(bad("node table length"));
    }
    let mut nodes = Vec::new();
    for n in tree.chunks_exact(32) {
        let u = |p| u32::from_le_bytes(n[p..p + 4].try_into().unwrap());
        nodes.push(Node {
            level: n[2],
            ty: table_string(st, u(4))?,
            name: table_string(st, u(8))?,
            size: u(12) as i32,
            flags: u(20),
            end: 0,
        });
    }
    for i in 0..nodes.len() {
        nodes[i].end = (i + 1..nodes.len())
            .find(|&j| nodes[j].level <= nodes[i].level)
            .unwrap_or(nodes.len());
    }
    let mut r = Reader {
        data,
        pos: 0,
        nodes: &nodes,
    };
    let v = r.node(0)?;
    if r.pos != data.len() {
        return Err(bad(&format!(
            "type-tree consumed {} of {} bytes",
            r.pos,
            data.len()
        )));
    }
    Ok(v)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn leaf(ty: u32, flags: u32) -> Vec<u8> {
        let mut n = vec![0; 32];
        n[4..8].copy_from_slice(&(ty | 0x80000000).to_le_bytes());
        n[8..12].copy_from_slice(&(55u32 | 0x80000000).to_le_bytes());
        n[12..16].copy_from_slice(&(-1i32).to_le_bytes());
        n[20..24].copy_from_slice(&flags.to_le_bytes());
        n
    }
    #[test]
    fn typeless_data_consumes_all_bytes_and_padding() {
        let n = leaf(894, 0x4000);
        let data = [3, 0, 0, 0, 7, 8, 9, 0];
        assert_eq!(read(&n, &[], &data).unwrap().bytes(), &[7, 8, 9]);
        assert!(read(&n, &[], &data[..7]).is_err());
    }
    #[test]
    fn signed_64bit_object_ids_are_preserved() {
        let n = leaf(814, 0);
        let id = -4_705_697_800_054_608_130i64;
        assert_eq!(read(&n, &[], &id.to_le_bytes()).unwrap().int(), id);
    }
    #[test]
    fn rejects_empty_unknown_and_truncated_trees() {
        assert!(read(&[], &[], &[]).is_err());
        assert!(read(&leaf(999999, 0), &[], &[]).is_err());
        assert!(read(&leaf(814, 0), &[], &[1, 2]).is_err());
    }
}
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    nodes: &'a [Node],
}
impl Reader<'_> {
    fn take(&mut self, n: usize) -> R<&[u8]> {
        let e = self
            .pos
            .checked_add(n)
            .ok_or_else(|| bad("object overflow"))?;
        let p = self.pos;
        self.pos = e;
        self.data.get(p..e).ok_or_else(|| bad("object truncated"))
    }
    fn node(&mut self, i: usize) -> R<Value> {
        let n = &self.nodes[i];
        let ty = n.ty.clone();
        let end = n.end;
        let flags = n.flags;
        let size = n.size;
        let v = match ty.as_str() {
            "TypelessData" => {
                let sz = i32::from_le_bytes(self.take(4)?.try_into()?);
                if !(0..=64_000_000).contains(&sz) {
                    return Err(bad("binary size"));
                }
                Value::Bytes(self.take(sz as usize)?.to_vec())
            }
            "string" => {
                let sz = i32::from_le_bytes(self.take(4)?.try_into()?);
                if !(0..=16_777_216).contains(&sz) {
                    return Err(bad("string size"));
                }
                let s = String::from_utf8(self.take(sz as usize)?.to_vec())?;
                self.pos = (self.pos + 3) & !3;
                Value::Text(s)
            }
            "Array" => {
                let len = i32::from_le_bytes(self.take(4)?.try_into()?);
                if !(0..=64_000_000).contains(&len) {
                    return Err(bad(&format!(
                        "array size {len} at byte {} node {i} {}",
                        self.pos - 4,
                        n.name
                    )));
                }
                let child = i + 2;
                let t = &self.nodes[child].ty;
                if self.nodes[child].end == child + 1
                    && matches!(t.as_str(), "UInt8" | "char" | "SInt8")
                {
                    Value::Bytes(self.take(len as usize)?.to_vec())
                } else {
                    let mut a = Vec::new();
                    for _ in 0..len {
                        a.push(self.node(child)?);
                    }
                    Value::Array(a)
                }
            }
            "bool" | "UInt8" | "char" => Value::Int(self.take(1)?[0] as i64),
            "SInt8" => Value::Int(self.take(1)?[0] as i8 as i64),
            "SInt16" | "short" => Value::Int(i16::from_le_bytes(self.take(2)?.try_into()?) as i64),
            "UInt16" | "unsigned short" => {
                Value::Int(u16::from_le_bytes(self.take(2)?.try_into()?) as i64)
            }
            "SInt32" | "int" => Value::Int(i32::from_le_bytes(self.take(4)?.try_into()?) as i64),
            "UInt32" | "unsigned int" => {
                Value::Int(u32::from_le_bytes(self.take(4)?.try_into()?) as i64)
            }
            "SInt64" | "long long" => Value::Int(i64::from_le_bytes(self.take(8)?.try_into()?)),
            "UInt64" | "unsigned long long" | "FileSize" => {
                Value::Int(u64::from_le_bytes(self.take(8)?.try_into()?) as i64)
            }
            "float" => Value::Float(f32::from_le_bytes(self.take(4)?.try_into()?) as f64),
            "double" => Value::Float(f64::from_le_bytes(self.take(8)?.try_into()?)),
            _ => {
                if end == i + 1 {
                    if size < 0 {
                        return Err(bad(&format!("unsupported variable leaf {ty}")));
                    }
                    Value::Bytes(self.take(size as usize)?.to_vec())
                } else if i + 1 < end && self.nodes[i + 1].ty == "Array" {
                    self.node(i + 1)?
                } else {
                    let mut fields = Vec::new();
                    let mut j = i + 1;
                    while j < end {
                        let key = self.nodes[j].name.clone();
                        fields.push((key, self.node(j)?));
                        j = self.nodes[j].end;
                    }
                    Value::Object(fields)
                }
            }
        };
        if flags & 0x4000 != 0 {
            self.pos = (self.pos + 3) & !3;
        }
        if self.pos > self.data.len() {
            return Err(bad("alignment exceeds object"));
        }
        Ok(v)
    }
}
