import {RequestImportMessages} from "./RequestImportMessages";
import {useEffect, useState} from "react";
import {ImportWarning, Collection} from "../bindings";
import {Button} from "primereact/button";

export type MessageMap = Map<string, ImportWarning[]>

interface ComponentProps {
    collection: Collection,
    importWarnings: ImportWarning[]

    // optional, if given we can clear the warnings with a button click, otherwise
    // we just display the warnings
    onClearWarnings: ((event: any) => void) | undefined
}

export function ImportResultComponent(props: ComponentProps) {
    const [requestToMessages, setRequestToMessages] = useState<MessageMap | undefined>(undefined);

    useEffect(() => {
        let messageMap = new Map<string, ImportWarning[]>()
    // @TODO check if this still works after migration
        props.importWarnings.map((importWarning: ImportWarning) => {
            /* @TODO migration let messages = messageMap.get(importWarning.RelativeRequestPath);
            if (!messages) {
                messages = [];
            }
            messages.push(importWarning);
            messageMap.set(importWarning.RelativeRequestPath, messages); */
        });
        setRequestToMessages(messageMap);
    }, [props.importWarnings]);

    return (
        <div style={{display: 'flex', flexDirection: 'column', alignItems: 'flex-start'}}>
            <p style={{marginTop: '20px', textAlign: 'left'}}>The collection has been imported but some problems
                were encountered during the import:</p>
            {
                props.onClearWarnings &&
                <Button severity={"warning"} label="Clear Warnings" icon="pi pi-check" onClick={props.onClearWarnings}
                        style={{marginTop: '20px', marginBottom: '10px'}}/>
            }

            {
                requestToMessages &&
                [...requestToMessages.keys()].map((relativeRequestPath: string) => {
                    return <RequestImportMessages
                                                  relativeRequestPath={relativeRequestPath}
                                                  messages={requestToMessages.get(relativeRequestPath) || []}/>
                })
            }
        </div>
    )
}
