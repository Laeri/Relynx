import { Accordion, AccordionTab } from "primereact/accordion";
import { ImportResultComponent } from "./ImportResultComponent";
import { Collection, ImportWarning } from '../bindings';
import { confirmPopup } from "primereact/confirmpopup";

interface ComponentProps {
  collection: Collection,
  onClearWarnings: () => void,
  importWarnings: ImportWarning[]
}

export function WarningCollapsible(props: ComponentProps) {

  const confirmClearImportWarnings = (event: any) => {
    confirmPopup({
      target: event.currentTarget,
      message: 'Are you sure you want to clear the import warnings?',
      icon: 'pi pi-exclamation-triangle',
      accept: props.onClearWarnings,
      reject: () => {
      },
    });
    event.stopImmediatePropagation();
    event.preventDefault();
  };
  return (
    <Accordion style={{ marginTop: '30px' }} className={"p-accordion-thin"}>
      <AccordionTab
        className={"wide-accordion-header"}
        header={
          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
            <div>
              <i className="pi pi-exclamation-triangle mr-2 color-warn"></i>
              <span
                className="vertical-align-middle color-warn">Import Problems</span>
            </div>
          </div>
        }
      >
        <ImportResultComponent
          collection={props.collection}
          importWarnings={props.importWarnings}
          onClearWarnings={confirmClearImportWarnings}
        />
      </AccordionTab>
    </Accordion>
  )
}
