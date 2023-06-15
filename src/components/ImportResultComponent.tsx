import { RequestImportMessages } from "./RequestImportMessages";
import { useEffect, useState } from "react";
import { ImportWarning, Collection } from "../bindings";
import { Button } from "primereact/button";
import { Accordion, AccordionTab } from "primereact/accordion";

export type MessageMap = Map<string, ImportWarning[]>

interface ComponentProps {
  collection: Collection,
  importWarnings: ImportWarning[],
  // optional, if given we can clear the warnings with a button click, otherwise
  // we just display the warnings
  onClearWarnings: ((event: any) => void) | undefined
}

export function ImportResultComponent(props: ComponentProps) {
  const [pathToMessages, setPathToMessages] = useState<MessageMap | undefined>(undefined);

  useEffect(() => {
    let messageMap = new Map<string, ImportWarning[]>()
    props.importWarnings.map((importWarning: ImportWarning) => {
      let messages = messageMap.get(importWarning.rest_file_path);
      if (!messages) {
        messages = [];
      }
      messages.push(importWarning);
      messageMap.set(importWarning.rest_file_path, messages);
    });
    setPathToMessages(messageMap);
  }, [props.collection.import_warnings]);



  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
      <p style={{ marginTop: '20px', textAlign: 'left' }}>The collection has been imported but some problems
        were encountered during the import:</p>
      {
        props.onClearWarnings &&
        <Button severity={"warning"} label="Clear Warnings" icon="pi pi-check" onClick={props.onClearWarnings}
          style={{ marginTop: '20px', marginBottom: '10px' }} />
      }

      {
        pathToMessages &&
        [...pathToMessages.keys()].map((requestPath: string) => {
          let relativeRequestPath = requestPath.replace(props.collection.path, '');
          let messages = pathToMessages.get(requestPath) as ImportWarning[]; // we know the list is not empty

          // remove extension for display in legend/header
          relativeRequestPath = relativeRequestPath.replace('.http', '').replace('.rest', '');

          let legend;
          if (messages[0].is_group) {
            legend = `Group: ${relativeRequestPath}`;
          } else {
            legend = `Request: ${relativeRequestPath}`;
          }

          return (
            <Accordion style={{ marginTop: '30px' }}>
              <AccordionTab header={legend}>
                <RequestImportMessages
                  collection={props.collection}
                  absolutePath={requestPath}
                  messages={pathToMessages.get(requestPath) || []} />
              </AccordionTab>
            </Accordion>
          )
        })
      }
    </div>
  )
}
