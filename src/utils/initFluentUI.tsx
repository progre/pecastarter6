import { registerIcons } from '@fluentui/react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import {
  faCheck,
  faChevronDown,
  faChevronUp,
  faSearch,
  faXmark,
  faWarning,
  faCircleInfo,
} from '@fortawesome/free-solid-svg-icons';
import {
  faCircleXmark,
  faClipboard,
  faFolder,
  faFolderOpen,
} from '@fortawesome/free-regular-svg-icons';
import { IconProp } from '@fortawesome/fontawesome-svg-core';
import { loadTheme, createTheme } from '@fluentui/react';

export default function initFluentUI() {
  // opt-out Segoe
  loadTheme(createTheme({ defaultFontStyle: { fontFamily: 'sans-serif' } }));

  // opt-out icons
  registerIcons({
    icons: {
      checkmark: <FontAwesomeIcon icon={faCheck} />,
      chevrondown: <FontAwesomeIcon icon={faChevronDown} />,
      chevrondownsmall: <FontAwesomeIcon icon={faChevronDown} />,
      chevronupsmall: <FontAwesomeIcon icon={faChevronUp} />,
      clear: <FontAwesomeIcon icon={faXmark} />,
      clipboard: <FontAwesomeIcon icon={faClipboard as IconProp} />,
      errorbadge: <FontAwesomeIcon icon={faCircleXmark as IconProp} />,
      folderopen: <FontAwesomeIcon icon={faFolderOpen as IconProp} />,
      folder: <FontAwesomeIcon icon={faFolder as IconProp} />,
      info: <FontAwesomeIcon icon={faCircleInfo} />,
      search: <FontAwesomeIcon icon={faSearch} />,
      warning: <FontAwesomeIcon icon={faWarning} />,
    },
  });
}
