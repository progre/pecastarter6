import { css } from '@emotion/react';
import { Pivot, PivotItem } from '@fluentui/react';
import { ReactNode, useState } from 'react';

interface TabContentProps {
  label: string;
  children?: ReactNode;
}

export function TabContent(props: TabContentProps) {
  return <>{props.children}</>;
}

export default function TabContainer(props: {
  initialTab?: string;
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

  const [current, setCurrent] = useState(
    props.initialTab ?? children[0].props.label ?? ''
  );
  return (
    <Pivot>
      {children?.map((x, i) => (
        <PivotItem key={i} headerText={x.props.label}>
          <div
            css={css`
              margin: 16px 8px 8px;
            `}
          >
            {x.props.children}
          </div>
        </PivotItem>
      ))}
    </Pivot>
  );
}
