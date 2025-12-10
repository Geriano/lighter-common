use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Validation error containing field-specific error messages
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Map of field names to their error messages
    pub errors: HashMap<String, Vec<String>>,
}

impl ValidationError {
    /// Create a new empty ValidationError
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    /// Add an error message for a specific field
    pub fn add(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors
            .entry(field.into())
            .or_insert_with(Vec::new)
            .push(message.into());
    }

    /// Check if there are any validation errors
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if there are any validation errors
    pub fn has_errors(&self) -> bool {
        !self.is_empty()
    }

    /// Get the total number of errors
    pub fn count(&self) -> usize {
        self.errors.values().map(|v| v.len()).sum()
    }

    /// Get errors for a specific field
    pub fn get_field_errors(&self, field: &str) -> Option<&Vec<String>> {
        self.errors.get(field)
    }

    /// Merge another ValidationError into this one
    pub fn merge(&mut self, other: ValidationError) {
        for (field, messages) in other.errors {
            self.errors
                .entry(field)
                .or_insert_with(Vec::new)
                .extend(messages);
        }
    }

    /// Create a ValidationError with a single field error
    pub fn single(field: impl Into<String>, message: impl Into<String>) -> Self {
        let mut error = Self::new();
        error.add(field, message);
        error
    }
}

impl Default for ValidationError {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "No validation errors");
        }

        write!(f, "Validation errors: ")?;
        for (i, (field, messages)) in self.errors.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: [{}]", field, messages.join(", "))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_validation_error() {
        let error = ValidationError::new();
        assert!(error.is_empty());
        assert_eq!(error.count(), 0);
    }

    #[test]
    fn test_add_error() {
        let mut error = ValidationError::new();
        error.add("email", "Email is required");
        error.add("password", "Password is too short");

        assert!(!error.is_empty());
        assert_eq!(error.count(), 2);
        assert_eq!(
            error.get_field_errors("email"),
            Some(&vec!["Email is required".to_string()])
        );
    }

    #[test]
    fn test_multiple_errors_same_field() {
        let mut error = ValidationError::new();
        error.add("password", "Password is too short");
        error.add("password", "Password must contain a number");

        assert_eq!(error.count(), 2);
        let password_errors = error.get_field_errors("password").unwrap();
        assert_eq!(password_errors.len(), 2);
    }

    #[test]
    fn test_single_constructor() {
        let error = ValidationError::single("name", "Name is required");
        assert_eq!(error.count(), 1);
        assert_eq!(
            error.get_field_errors("name"),
            Some(&vec!["Name is required".to_string()])
        );
    }

    #[test]
    fn test_merge() {
        let mut error1 = ValidationError::new();
        error1.add("email", "Email is required");

        let mut error2 = ValidationError::new();
        error2.add("password", "Password is required");

        error1.merge(error2);

        assert_eq!(error1.count(), 2);
        assert!(error1.get_field_errors("email").is_some());
        assert!(error1.get_field_errors("password").is_some());
    }

    #[test]
    fn test_display() {
        let mut error = ValidationError::new();
        error.add("email", "Email is invalid");
        error.add("password", "Password is too short");

        let display = error.to_string();
        assert!(display.contains("email"));
        assert!(display.contains("password"));
    }
}
