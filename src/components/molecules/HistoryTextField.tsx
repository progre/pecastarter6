import { ComboBox } from '@fluentui/react';

export default function HistoryTextField(props: {
  label: string;
  required?: boolean;
  value: string;
  placeholder?: string;
  history: string[];
  onChange: (value: string) => void;
}) {
  return (
    <ComboBox
      label={props.label}
      required={props.required}
      allowFreeform
      placeholder={props.placeholder}
      options={props.history.map((text, key) => ({ key, text }))}
      text={props.value}
      onItemClick={(_e, option, _i) => props.onChange(option!!.text)}
      onInputValueChange={(value) => props.onChange(value)}
    />
  );
}
