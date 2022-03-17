import { css } from '@emotion/react';
import {
  ChangeEventHandler,
  InputHTMLAttributes,
  useEffect,
  useRef,
} from 'react';

export default function TextField(props: {
  label: string;
  type: InputHTMLAttributes<never>['type'];
  max?: InputHTMLAttributes<never>['max'];
  min?: InputHTMLAttributes<never>['min'];
  placeholder?: string;
  required?: boolean;
  disabled?: boolean;
  value?: string | number;
  history?: readonly string[];
  fitContent?: boolean;

  onBlur?: ChangeEventHandler<HTMLDivElement>;
  onChangeValue?: (value: string) => void;
  onChangeValueAsNumber?: (value: number) => void;
}) {
  const inputId = `_${(Math.random() * Number.MAX_SAFE_INTEGER) | 0}`;
  const ref = useRef<HTMLInputElement>(null);
  return (
    <div
      css={css`
        display: flex;
        flex-direction: column;
      `}
    >
      <label
        htmlFor={inputId}
        css={css`
          color: ${props.disabled ? 'lightgray' : 'inherit'};
        `}
      >
        {props.label}
      </label>
      <div
        css={css`
          display: flex;
        `}
      >
        <input
          ref={ref}
          id={inputId}
          type={props.type}
          placeholder={props.placeholder}
          value={props.value}
          required={props.required}
          min={props.min}
          max={props.max}
          disabled={props.disabled}
          style={props.fitContent ? { width: 'fit-content' } : {}}
          onChange={(e) => {
            props.onChangeValue?.(e.target.value);
            props.onChangeValueAsNumber?.(e.target.valueAsNumber);
          }}
          css={css`
            width: 100%;
          `}
        />
        {props.history == null ? null : (
          <>
            <select
              css={css`
                display: flex;
                text-align: right;
                width: 3em;
              `}
              value=""
              onChange={(e) => {
                ref.current!!.focus();
                props.onChangeValue?.(e.target.value);
              }}
            >
              <option>...</option>
              {props.history.map((x, i) => (
                <option key={i}>{x}</option>
              ))}
            </select>
          </>
        )}
      </div>
    </div>
  );
}
