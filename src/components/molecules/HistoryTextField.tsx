import { ComboBox } from '@fluentui/react';
import { useState } from 'react';

export default function HistoryTextField(props: {
  label: string;
  required?: boolean;
  value: string;
  placeholder?: string;
  history: string[];
  onChange: (value: string) => void;
}) {
  const [value, setValue] = useState(props.value);
  return (
    <ComboBox
      label={props.label}
      required={props.required}
      allowFreeform
      placeholder={props.placeholder}
      options={[...props.history].reverse().map((x) => ({ key: x, text: x }))}
      styles={{
        optionsContainer: {
          display: 'flex',
          flexDirection: 'column-reverse',
        },
      }}
      text={value}
      onItemClick={(_e, option, _i) => setValue(option!!.text)}
      onInputValueChange={(value) => setValue(value)}
      onBlurCapture={() => {
        if (value === props.value) {
          return;
        }
        props.onChange(value);
      }}
    />
  );
}
