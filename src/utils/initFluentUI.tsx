import { registerIcons } from '@fluentui/react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import {
  faCheck,
  faChevronDown,
  faChevronUp,
  faSearch,
  faXmark,
  faWarning,
  faInfo,
  faCircleInfo,
} from '@fortawesome/free-solid-svg-icons';
import {
  faCircleXmark,
  faCopy,
  faFolder,
  faFolderOpen,
} from '@fortawesome/free-regular-svg-icons';
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
      copy: <FontAwesomeIcon icon={faCopy} />,
      errorbadge: <FontAwesomeIcon icon={faCircleXmark} />,
      folderopen: <FontAwesomeIcon icon={faFolderOpen} />,
      folder: <FontAwesomeIcon icon={faFolder} />,
      info: <FontAwesomeIcon icon={faCircleInfo} />,
      search: <FontAwesomeIcon icon={faSearch} />,
      warning: <FontAwesomeIcon icon={faWarning} />,
    },
  });
}
