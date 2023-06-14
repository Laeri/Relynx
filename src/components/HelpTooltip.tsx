import { Tooltip } from "primereact/tooltip";
import { newUUID } from "../model/request"

interface ComponentProps {
  text: string,
  style?: any
}

export function HelpTooltip(props: ComponentProps) {
  const tooltipId = `tooltip-${newUUID()}`;
  return (
    <>
      <Tooltip target={`#${tooltipId}`} />

      <i id={tooltipId} className="pi pi-question-circle p-text-secondary"
        data-pr-tooltip={props.text}
        data-pr-position="right"
        data-pr-at="right+5 top"
        data-pr-my="left center-2"
        style={{ fontSize: '1rem', cursor: 'pointer', ...(props.style ?? {}) }}>
      </i>
    </>

  )
}
