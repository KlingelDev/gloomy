use serde::{Serialize, Deserialize};
use regex::Regex;

/// Rule defining validation logic.
// We can't easily serialize closures/Box<dyn Fn>, so the custom logic might need to be application-side or trait objects if we want full serialization.
// For Gloomy's declarative nature, we might stick to common patterns and maybe a "Custom" validation *func name* or similar if using RON.
// But for code usage, Box<dyn Fn> is fine.
// Since Widget struct is Serialize/Deserialize, we can't put non-serializable things directly in validatable if we want to serialize Widget.
// NOTE: Widget enum derives Serialize/Deserialize. So Custom rules with closures are problematic if stored IN the widget tree directly.
//
// OPTION 1: Separate ValidationStore? (Complex)
// OPTION 2: Only support serializable rules (Min, Max, Regex) and let app handle complex logic alongside?
// OPTION 3: Skip serialization for Custom rules? (But we need to store them)
//
// Decision: For now, support only Serializable rules. Complex validation is usually business logic done in the app anyway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Min(f64),
    Max(f64),
    Pattern(String), // Regex pattern as string
    Email,
}

impl ValidationRule {
    pub fn validate(&self, value: &str) -> Result<(), String> {
        if value.is_empty() {
             if matches!(self, ValidationRule::Required) {
                 return Err("This field is required".to_string());
             }
             return Ok(()); // Empty non-required is valid usually?
        }
        
        match self {
            ValidationRule::Required => {
                // Already checked empty
                Ok(())
            }
            ValidationRule::MinLength(len) => {
                if value.len() < *len {
                     Err(format!("Must be at least {} characters", len))
                } else {
                     Ok(())
                }
            }
            ValidationRule::MaxLength(len) => {
                if value.len() > *len {
                     Err(format!("Must be no more than {} characters", len))
                } else {
                     Ok(())
                }
            }
            ValidationRule::Min(min) => {
                if let Ok(num) = value.parse::<f64>() {
                     if num < *min {
                         Err(format!("Must be at least {}", min))
                     } else {
                         Ok(())
                     }
                } else {
                     Err("Invalid number".to_string())
                }
            }
            ValidationRule::Max(max) => {
                if let Ok(num) = value.parse::<f64>() {
                     if num > *max {
                         Err(format!("Must be no more than {}", max))
                     } else {
                         Ok(())
                     }
                } else {
                     Err("Invalid number".to_string())
                }
            }
            ValidationRule::Pattern(pat) => {
                // Regex compilation every time is slow. 
                // In a real system we'd cache this.
                if let Ok(re) = Regex::new(pat) {
                    if re.is_match(value) {
                         Ok(())
                    } else {
                         Err("Invalid format".to_string())
                    }
                } else {
                    Err("Invalid pattern configuration".to_string())
                }
            }
            ValidationRule::Email => {
                // Simple email regex
                 if let Ok(re) = Regex::new(r"^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$") {
                    if re.is_match(value) {
                         Ok(())
                    } else {
                         Err("Invalid email address".to_string())
                    }
                } else {
                    Err("Regex error".to_string())
                }
            }
        }
    }
}
