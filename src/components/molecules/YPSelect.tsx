import { css } from '@emotion/react';
import { Dropdown, ResponsiveMode } from '@fluentui/react';
import YPConfig from '../../entities/YPConfig';

export default function YPSelect(props: {
  label: string;
  ypConfigs: readonly YPConfig[];
  host: string;
  usedHostForIPV4?: string;
  conflict: boolean;
  onChange(host: string): void;
}): JSX.Element {
  return (
    <Dropdown
      css={css`
        display: flex;
        align-items: center;
        > div {
          margin-left: 8px;
          flex-grow: 1;
        }
      `}
      label={props.label}
      responsiveMode={ResponsiveMode.large}
      onRenderTitle={(innerProps, defaultRender) => (
        <div
          css={css`
            color: ${!props.conflict ? 'inherit' : '#ff2800'};
          `}
        >
          {defaultRender!!(innerProps)}
        </div>
      )}
      onRenderItem={(item, defaultRender) => (
        <div
          key={item!!.key}
          css={css`
            button {
              color: ${item!!.data.host !== props.usedHostForIPV4
                ? 'initial'
                : '#ff2800'};
            }
          `}
        >
          {defaultRender!!(item)}
        </div>
      )}
      options={[
        { key: -1, text: '掲載しない', data: { host: '' } },
        ...props.ypConfigs.map((x, i) => ({
          key: i,
          text: x.name,
          data: { host: x.host },
        })),
      ]}
      selectedKey={props.ypConfigs.findIndex((x) => x.host === props.host)}
      onChange={(_e, option) =>
        props.onChange(props.ypConfigs[Number(option?.key)]?.host ?? '')
      }
    />
  );
}
