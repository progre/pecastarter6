import { registerIcons } from '@fluentui/react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import {
  faCheck,
  faChevronDown,
  faChevronUp,
  faCopy,
} from '@fortawesome/free-solid-svg-icons';
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
      copy: <FontAwesomeIcon icon={faCopy} />,
    },
  });
}