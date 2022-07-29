
use wildmatch::WildMatch;

pub struct Exclude {
    globs: Vec<WildMatch>,
}

impl Exclude {
    pub fn new(args: Vec<String>) -> Self {
        Self { globs: args.iter().map(|e| WildMatch::new(&e)).collect() }
    }

    pub fn matches(&self, s: &str) -> bool {
        self.globs.iter().position(|p| p.matches(s)).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::Exclude;

    #[test]
    fn empty() {
        let ex = Exclude::new(vec![]);
        assert_eq!(ex.matches("ABC"), false);
    }

    #[test]
    fn single_pattern() {
        let ex = Exclude::new(vec![".*".to_owned()]);
        assert_eq!(ex.matches(".DS_STORE"), true);
        assert_eq!(ex.matches("A.exe"), false);
    }
    #[test]
    fn few_patterns() {
        let ex = Exclude::new(vec![".*".to_owned(), "*~".to_owned()]);
        assert_eq!(ex.matches(".DS_STORE"), true);
        assert_eq!(ex.matches("A~"), true);
        assert_eq!(ex.matches("A.exe"), false);
    }
}