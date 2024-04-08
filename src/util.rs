use uuid::Uuid;

pub fn new_id(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_id() {
        let id = new_id("node");
        assert_eq!(id.len(), 4 + 1 + 36);
        assert_eq!(&id[0..4], "node");
    }
}
