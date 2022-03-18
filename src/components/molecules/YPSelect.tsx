import { css } from '@emotion/react';
import YPConfig from '../../entities/YPConfig';

export default function YPSelect(props: {
  id: string;
  ypConfigs: readonly YPConfig[];
  host: string;
  usedHostForIPV4?: string;
  conflict: boolean;
  onChange(host: string): void;
}): JSX.Element {
  return (
    <select
      id={props.id}
      css={css`
        flex-grow: 1;
        color: ${!props.conflict ? 'inherit' : '#ff2800'};
      `}
      value={props.ypConfigs.findIndex((x) => x.host === props.host)}
      onChange={(e) =>
        props.onChange(props.ypConfigs[Number(e.target.value)]?.host ?? '')
      }
    >
      <option
        value={-1}
        css={css`
          color: initial;
        `}
      >
        掲載しない
      </option>
      {props.ypConfigs.map((x, i) => (
        <option
          key={i}
          value={i}
          css={css`
            color: ${x.host !== props.usedHostForIPV4 ? 'initial' : '#ff2800'};
          `}
        >
          {x.name}
        </option>
      ))}
    </select>
  );
}
