const BACKEND_PREFIX =
  import.meta.env.VITE_BACKEND_PREFIX ?? window.location.origin;

const DEFAULT_PROFILE_PICTURE_URL =
  BACKEND_PREFIX + '/public/assets/default-avatar.png';

const TEMPLATE_PARAMS = {
  separator: '-',
  siteName: 'Kitsune',
};

const TITLE_TEMPLATE = '%s %separator %siteName';

export {
  BACKEND_PREFIX,
  DEFAULT_PROFILE_PICTURE_URL,
  TEMPLATE_PARAMS,
  TITLE_TEMPLATE,
};
