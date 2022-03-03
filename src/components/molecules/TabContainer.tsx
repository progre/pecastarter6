import { css } from '@emotion/react';
import { ReactNode, useState } from 'react';

interface TabContentProps {
  label: string;
  children?: ReactNode;
}

export function TabContent(props: TabContentProps) {
  return <>{props.children}</>;
}

export default function TabContainer(props: {
  children?: { props: TabContentProps } | { props: TabContentProps }[];
}): JSX.Element {
  let children: readonly { props: TabContentProps }[];
  if (props.children == null) {
    children = [];
  } else if (!Array.isArray(props.children)) {
    children = [props.children];
  } else {
    children = props.children;
  }

  const [current, setCurrent] = useState(children[0].props.label ?? '');
  return (
    <div
      css={css`
        user-select: none;
      `}
    >
      <div role="tablist">
        {children?.map((x, i) => (
          <button
            key={x.props.label}
            role="tab"
            tabIndex={i}
            onClick={() => setCurrent(x.props.label)}
          >
            {x.props.label}
          </button>
        ))}
      </div>
      <div
        css={css`
          margin: 8px;
          user-select: none;
        `}
      >
        {children?.map((x, i) => (
          <div
            key={x.props.label}
            role="tabpanel"
            tabIndex={i}
            style={current === x.props.label ? {} : { display: 'none' }}
          >
            {x.props.children}
          </div>
        ))}
      </div>
    </div>
  );
}
