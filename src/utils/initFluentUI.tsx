import { registerIcons } from '@fluentui/react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faCheck, faChevronDown } from '@fortawesome/free-solid-svg-icons';
import { loadTheme, createTheme } from '@fluentui/react';

export default function initFluentUI() {
  // opt-out Segoe
  loadTheme(createTheme({ defaultFontStyle: { fontFamily: 'sans-serif' } }));

  // opt-out icons
  registerIcons({
    icons: {
      Checkmark: <FontAwesomeIcon icon={faCheck} />,
      Chevrondown: <FontAwesomeIcon icon={faChevronDown} />,
    },
  });
}
