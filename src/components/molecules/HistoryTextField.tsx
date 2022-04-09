import { ComboBox, IComboBox } from '@fluentui/react';
import { DOMAttributes } from 'react';

export default function HistoryTextField(props: {
  label: string;
  required?: boolean;
  value: string;
  placeholder?: string;
  history: string[];
  onChange: (value: string) => void;
  onBlurCapture?: DOMAttributes<IComboBox>['onBlurCapture'];
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
      onBlurCapture={props.onBlurCapture}
    />
  );
}
