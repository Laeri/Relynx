import {Dialog} from "primereact/dialog";
import {Button} from "primereact/button";
import {InputText} from "primereact/inputtext";
import {useState} from "react";
import {onCtrlEnter} from "../../common/eventhandling";

interface ComponentProps {
    isOpen: boolean
    onResolve: (requestName?: string) => void
    onReject: () => void
}

export function CreateRequestModal(props: ComponentProps) {

    const [requestName, setRequestName] = useState<string>("");

    const resolveRequestName = () => props.onResolve(requestName)
    return (
        <Dialog header="Create Request" visible={props.isOpen} dismissableMask={false}
                style={{width: '50vw'}}
                onHide={props.onResolve}
                footer={
                    <div>
                        <Button label="Cancel" icon="pi pi-times" className={'p-button-secondary p-button-text'}
                                onClick={() => props.onResolve()}/>
                        <Button label="Create" icon="pi pi-check"
                                onClick={resolveRequestName}
                                style={{marginLeft: '80px'}}/>
                    </div>
                }>
            <div style={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'flex-start',
                marginBottom: '50px',
                marginTop: '40px',
                flexGrow: 1
            }}>
                <div style={{display: 'flex', width: '100%', marginBottom: '20px', alignItems: 'center'}}>
                    <h3>Name</h3>
                    <InputText autoFocus={true} onKeyPress={(event: any) => onCtrlEnter(event, resolveRequestName)}
                               value={requestName} onChange={(e) => setRequestName(e.target.value)}
                               style={{marginLeft: '20px', flexBasis: '60%'}}/>
                </div>
            </div>

        </Dialog>
    )
}
