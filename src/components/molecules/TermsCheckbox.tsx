import { css } from '@emotion/css';
import { Checkbox } from '@fluentui/react';

export default function TermsCheckbox(props: {
  termsURL: string | null;
  readed: boolean;
  agreed: boolean;
  onClickReadTerms(): void;
  onChangeAgreeTerms(value: boolean): void;
}): JSX.Element {
  return (
    <Checkbox
      onRenderLabel={() => (
        <div
          className={css`
            margin-left: 4px;
          `}
        >
          <a
            href={props.termsURL == null ? undefined : ''}
            onClick={async (e) => {
              e.preventDefault();
              props.onClickReadTerms();
            }}
          >
            規約
          </a>
          <span
            style={{
              marginLeft: '0.25em',
              cursor: 'default',
              pointerEvents: 'none',
            }}
          >
            を確認し、同意した
          </span>
        </div>
      )}
      disabled={props.termsURL == null || (!props.agreed && !props.readed)}
      checked={props.agreed}
      onChange={(_e, checked) => props.onChangeAgreeTerms(checked == true)}
    />
  );
}
