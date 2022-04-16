import { css } from '@emotion/react';
import { ComboBox } from '@fluentui/react';
import { useRef, useState } from 'react';

export default function HistoryTextField(props: {
  label: string;
  required?: boolean;
  value: string;
  placeholder?: string;
  history: string[];
  onChange: (value: string) => void;
}) {
  const [value, setValue] = useState(props.value);
  const ref = useRef<HTMLInputElement>(null);
  return (
    <ComboBox
      ref={ref}
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
      selectedKey={null}
      text={value}
      onRenderList={(props, defaultRender) => (
        <div style={{ width: ref.current!!.clientWidth - 30 - 2 }}>
          {defaultRender!!(props)}
        </div>
      )}
      onRenderItem={(props, defaultRender) => (
        <div
          key={props?.key}
          css={
            props?.key !== value
              ? null
              : css`
                  > button,
                  > button:hover {
                    background-color: rgb(237, 235, 233);
                  }
                `
          }
        >
          {defaultRender!!(props)}
        </div>
      )}
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
