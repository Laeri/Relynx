import { Message } from "primereact/message";
import { Collection, ImportWarning, MessageSeverity } from "../bindings";

interface ComponentProps {
  absolutePath: string,
  messages: ImportWarning[],
  // if true they are rendered in a collapsible as there are probably multiple warnings, otherwise directly display
  collection: Collection
}

export function RequestImportMessages(props: ComponentProps) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
      {
        props.messages.map((message: ImportWarning) => {
          return <Message style={{ marginTop: '20px' }}
            severity={message.severity as MessageSeverity}
            text={message.message} />
        })
      }
    </div>
  )
}
