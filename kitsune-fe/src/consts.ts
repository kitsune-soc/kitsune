export const BACKEND_PREFIX =
  import.meta.env.VITE_BACKEND_PREFIX ?? window.location.origin;

export const DEFAULT_PROFILE_PICTURE_URL =
  BACKEND_PREFIX + '/public/assets/default-avatar.png';
