const BACKEND_PREFIX =
  import.meta.env.VITE_BACKEND_PREFIX ?? window.location.origin;

const DEFAULT_PROFILE_PICTURE_URL =
  BACKEND_PREFIX + '/public/default-avatar.png';

const MAX_UUID = 'FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF';

const TEMPLATE_PARAMS = {
  separator: '-',
  siteName: 'Kitsune',
};

const TITLE_TEMPLATE = '%s %separator %siteName';

export {
  BACKEND_PREFIX,
  DEFAULT_PROFILE_PICTURE_URL,
  MAX_UUID,
  TEMPLATE_PARAMS,
  TITLE_TEMPLATE,
};
