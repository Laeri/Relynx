import {Message} from "primereact/message";
import {ImportWarning} from "../bindings";
import {MessageSeverity} from "primereact/api";
import {Fieldset} from "primereact/fieldset";

interface ComponentProps {
    relativeRequestPath: string,
    messages: ImportWarning[]
}

export function RequestImportMessages(props: ComponentProps) {

    return (
        <Fieldset toggleable={true} legend={`Request: ${props.messages[0].RequestName}`} style={{
            marginTop: '30px',
            width: '100%',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'flex-start'
        }}>
            <div style={{display: 'flex', flexDirection: 'column', alignItems: 'flex-start'}}>
                {
                    `/${props.messages[0].RequestName}` !== props.relativeRequestPath &&
                    <span>Path: {props.relativeRequestPath}</span>
                }
                {
                    props.messages.map((message: ImportWarning) => {
                        return <Message style={{marginTop: '20px'}}
                                        severity={message.Severity as MessageSeverity}
                                        text={message.Message}/>
                    })
                }
            </div>

        </Fieldset>
    )
}
