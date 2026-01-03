//! Programming dictionary for tech term recognition.
//!
//! Converts commonly mis-transcribed programming terms to their correct forms:
//! - "A P I" → "API"
//! - "jason" → "JSON"
//! - "sequel" → "SQL"

use regex::Regex;
use std::sync::LazyLock;

/// Programming terms: (spoken pattern, correct form)
/// Patterns are case-insensitive.
const PROGRAMMING_TERMS: &[(&str, &str)] = &[
    // Acronyms (spaced letters) - LONGER PATTERNS FIRST to avoid partial matches
    ("H T T P S", "HTTPS"),
    ("h t t p s", "HTTPS"),
    ("H T T P", "HTTP"),
    ("h t t p", "HTTP"),
    ("A P I", "API"),
    ("a p i", "API"),
    ("U R L", "URL"),
    ("u r l", "URL"),
    ("U R I", "URI"),
    ("u r i", "URI"),
    ("U I", "UI"),
    ("u i", "UI"),
    ("U X", "UX"),
    ("u x", "UX"),
    ("C L I", "CLI"),
    ("c l i", "CLI"),
    ("G U I", "GUI"),
    ("g u i", "GUI"),
    ("I D", "ID"),
    ("i d", "ID"),
    ("O S", "OS"),
    ("o s", "OS"),
    ("D B", "DB"),
    ("d b", "DB"),
    ("S S H", "SSH"),
    ("s s h", "SSH"),
    ("S S L", "SSL"),
    ("s s l", "SSL"),
    ("T L S", "TLS"),
    ("t l s", "TLS"),
    ("S Q L", "SQL"),
    ("s q l", "SQL"),
    ("R E S T", "REST"),
    ("r e s t", "REST"),
    ("C S S", "CSS"),
    ("c s s", "CSS"),
    ("H T M L", "HTML"),
    ("h t m l", "HTML"),
    ("J S", "JS"),
    ("j s", "JS"),
    ("T S", "TS"),
    ("t s", "TS"),
    ("N P M", "npm"),
    ("n p m", "npm"),
    ("C I", "CI"),
    ("c i", "CI"),
    ("C D", "CD"),
    ("c d", "CD"),
    ("P R", "PR"),
    ("p r", "PR"),
    ("A W S", "AWS"),
    ("a w s", "AWS"),
    ("G C P", "GCP"),
    ("g c p", "GCP"),
    // Common mis-transcriptions
    ("jason", "JSON"),
    ("Jay son", "JSON"),
    ("sequel", "SQL"),
    ("my sequel", "MySQL"),
    ("post gres", "PostgreSQL"),
    ("postgres", "PostgreSQL"),
    ("mongo db", "MongoDB"),
    ("mongo D B", "MongoDB"),
    ("read is", "Redis"),
    ("redis", "Redis"),
    ("dock er", "Docker"),
    ("kubernetes", "Kubernetes"),
    ("K 8 S", "k8s"),
    ("k 8 s", "k8s"),
    ("kates", "k8s"),
    // Frameworks and languages
    ("type script", "TypeScript"),
    ("java script", "JavaScript"),
    ("node js", "Node.js"),
    ("node J S", "Node.js"),
    ("react js", "React"),
    ("react J S", "React"),
    ("next js", "Next.js"),
    ("next J S", "Next.js"),
    ("view js", "Vue.js"),
    ("vue js", "Vue.js"),
    ("vue J S", "Vue.js"),
    ("angular js", "Angular"),
    ("tailwind", "Tailwind"),
    ("tailwind css", "Tailwind CSS"),
    ("graph Q L", "GraphQL"),
    ("graph ql", "GraphQL"),
    // Common tools
    ("git hub", "GitHub"),
    ("git lab", "GitLab"),
    ("bit bucket", "Bitbucket"),
    ("vs code", "VS Code"),
    ("v s code", "VS Code"),
    ("x code", "Xcode"),
    ("intellij", "IntelliJ"),
    ("pie charm", "PyCharm"),
    ("pycharm", "PyCharm"),
    ("web storm", "WebStorm"),
    // Common programming terms
    ("async", "async"),
    ("a sync", "async"),
    ("await", "await"),
    ("a wait", "await"),
    ("null", "null"),
    ("nil", "nil"),
    ("boolean", "boolean"),
    ("bool", "bool"),
    ("string", "string"),
    ("integer", "integer"),
    ("int", "int"),
    ("float", "float"),
    ("double", "double"),
    ("array", "array"),
    ("object", "object"),
    ("function", "function"),
    ("method", "method"),
    ("class", "class"),
    ("interface", "interface"),
    ("enum", "enum"),
    ("struct", "struct"),
    ("tuple", "tuple"),
    ("hash map", "HashMap"),
    ("hash set", "HashSet"),
    ("vector", "Vec"),
    ("vec", "Vec"),
    // Git commands
    ("git init", "git init"),
    ("git clone", "git clone"),
    ("git commit", "git commit"),
    ("git push", "git push"),
    ("git pull", "git pull"),
    ("git merge", "git merge"),
    ("git rebase", "git rebase"),
    ("git checkout", "git checkout"),
    ("git branch", "git branch"),
    ("git stash", "git stash"),
    ("git diff", "git diff"),
    ("git log", "git log"),
    ("git status", "git status"),
];

/// Compiled regex patterns for dictionary replacement.
/// Uses LazyLock for thread-safe lazy initialization.
static DICTIONARY_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    PROGRAMMING_TERMS
        .iter()
        .filter_map(|(pattern, replacement)| {
            // Create case-insensitive word-boundary regex
            let regex_pattern = format!(r"(?i)\b{}\b", regex::escape(pattern));
            Regex::new(&regex_pattern).ok().map(|re| (re, *replacement))
        })
        .collect()
});

/// Apply programming dictionary to text.
///
/// Replaces commonly mis-transcribed programming terms with their correct forms.
///
/// # Examples
/// ```ignore
/// let result = apply_dictionary("Use the A P I to fetch jason data");
/// assert_eq!(result, "Use the API to fetch JSON data");
/// ```
pub fn apply_dictionary(text: &str) -> String {
    let mut result = text.to_string();

    for (regex, replacement) in DICTIONARY_PATTERNS.iter() {
        result = regex.replace_all(&result, *replacement).to_string();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_spaced() {
        assert_eq!(apply_dictionary("Use the A P I"), "Use the API");
    }

    #[test]
    fn test_json_misspelling() {
        assert_eq!(apply_dictionary("Return jason data"), "Return JSON data");
    }

    #[test]
    fn test_sql_misspelling() {
        assert_eq!(apply_dictionary("Run a sequel query"), "Run a SQL query");
    }

    #[test]
    fn test_multiple_terms() {
        let result = apply_dictionary("Use the A P I to fetch jason from the D B");
        assert_eq!(result, "Use the API to fetch JSON from the DB");
    }

    #[test]
    fn test_http_https() {
        assert_eq!(apply_dictionary("H T T P request"), "HTTP request");
        assert_eq!(apply_dictionary("H T T P S endpoint"), "HTTPS endpoint");
    }

    #[test]
    fn test_frameworks() {
        assert_eq!(apply_dictionary("Using type script"), "Using TypeScript");
        assert_eq!(apply_dictionary("node js server"), "Node.js server");
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(apply_dictionary("use the a p i"), "use the API");
    }

    #[test]
    fn test_git_commands() {
        assert_eq!(apply_dictionary("Run git commit"), "Run git commit");
    }

    #[test]
    fn test_preserves_non_matches() {
        assert_eq!(apply_dictionary("Hello world"), "Hello world");
    }

    #[test]
    fn test_kubernetes() {
        assert_eq!(apply_dictionary("Deploy to K 8 S"), "Deploy to k8s");
    }
}
