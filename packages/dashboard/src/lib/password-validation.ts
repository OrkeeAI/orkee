// ABOUTME: Password strength validation utilities
// ABOUTME: Validates passwords against security requirements matching backend

const MIN_PASSWORD_LENGTH = 8;

export interface PasswordValidationResult {
  valid: boolean;
  error?: string;
}

/**
 * Validates password meets security requirements
 * - Minimum 8 characters (longer is better for Argon2)
 * - At least one uppercase letter
 * - At least one lowercase letter
 * - At least one digit
 * - At least one special character
 */
export function validatePasswordStrength(password: string): PasswordValidationResult {
  if (password.length < MIN_PASSWORD_LENGTH) {
    return {
      valid: false,
      error: `Password must be at least ${MIN_PASSWORD_LENGTH} characters long`,
    };
  }

  const hasUppercase = /[A-Z]/.test(password);
  const hasLowercase = /[a-z]/.test(password);
  const hasDigit = /\d/.test(password);
  const hasSpecial = /[^A-Za-z0-9]/.test(password);

  if (!hasUppercase) {
    return {
      valid: false,
      error: 'Password must contain at least one uppercase letter',
    };
  }

  if (!hasLowercase) {
    return {
      valid: false,
      error: 'Password must contain at least one lowercase letter',
    };
  }

  if (!hasDigit) {
    return {
      valid: false,
      error: 'Password must contain at least one digit',
    };
  }

  if (!hasSpecial) {
    return {
      valid: false,
      error: 'Password must contain at least one special character',
    };
  }

  return { valid: true };
}
