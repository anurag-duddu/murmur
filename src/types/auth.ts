/**
 * Authentication types for WorkOS integration.
 */

export interface UserInfo {
  id: string;
  email: string;
  first_name?: string;
  last_name?: string;
  profile_picture_url?: string;
}

export interface AuthState {
  is_authenticated: boolean;
  user: UserInfo | null;
  is_loading: boolean;
}
