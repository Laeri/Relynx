import { Message } from "primereact/message";
import { ImportWarning, MessageSeverity } from "../bindings";
import { Fieldset } from "primereact/fieldset";
import { Accordion, AccordionTab } from "primereact/accordion";

interface ComponentProps {
  relativeRequestPath: string,
  messages: ImportWarning[]
}

export function RequestImportMessages(props: ComponentProps) {

  const legend = `Request: ${props.messages[0].node_name ?? props.messages[0].rest_file_path}`;
  return (
    <Accordion activeIndex={0}
       style={{
        marginTop: '20px'
      }}>
      <AccordionTab header={legend}>
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
          {
            `/${props.messages[0].node_name ?? props.messages[0].rest_file_path}` !== props.relativeRequestPath &&
            <span>Path: {props.relativeRequestPath}</span>
          }
          {
            props.messages.map((message: ImportWarning) => {
              return <Message style={{ marginTop: '20px' }}
                severity={message.severity as MessageSeverity}
                text={message.message} />
            })
          }
        </div>
      </AccordionTab>

    </Accordion>
  )
}
