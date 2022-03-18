import { css } from '@emotion/react';

export default function TermsCheckbox(props: {
  termsURL: string | null;
  readed: boolean;
  agreed: boolean;
  onClickReadTerms(): void;
  onChangeAgreeTerms(value: boolean): void;
}): JSX.Element {
  const id = `_${(Math.random() * Number.MAX_SAFE_INTEGER) | 0}`;

  return (
    <div>
      <input
        id={id}
        type="checkbox"
        disabled={props.termsURL == null || (!props.agreed && !props.readed)}
        checked={props.agreed}
        onChange={(e) => props.onChangeAgreeTerms(e.target.checked)}
      />
      <label
        htmlFor={id}
        css={css`
          padding-left: 0.25em;
          color: ${props.termsURL == null ? 'lightgray' : 'inherit'};
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
        </a>{' '}
        を確認し、同意した
      </label>
    </div>
  );
}
