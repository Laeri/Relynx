import {Dialog} from "primereact/dialog";
import {Button} from "primereact/button";
import {InputText} from "primereact/inputtext";
import {useContext, useState} from "react";
import {backend} from "../../rpc";
import {ToastContext} from "../../App";
import {catchError} from "../../common/errorhandling";

interface ComponentProps {
    isOpen: boolean
    onResolve: (result?: { collectionName: string, collectionPath: string, importCollectionFilepath: string }) => void
    onReject: () => void
}

export function ImportCollectionModal(props: ComponentProps) {

    const [collectionName, _setCollectionName] = useState<string>("");
    const [collectionPath, setCollectionPath] = useState<string>("");
    const [importCollectionFilepath, setImportCollectionFilepath] = useState<string>("");

    const toast = useContext(ToastContext);

    const importCollectionFilePicker = () => {
        backend.selectFile()
            .then((result: string) => {
                setImportCollectionFilepath(result);
            })
            .catch(catchError(toast));
    }

    const openCollectionDirectoryPicker = () => {
        backend.selectDirectory()
            .then((result: string) => {
                setCollectionPath(result);
            })
            .catch(catchError(toast));
    }

    return (
        <Dialog header="Import Collection" visible={props.isOpen} dismissableMask={false}
                style={{width: '50vw'}}
                onHide={() => props.onResolve()}
                footer={
                    <div>
                        <Button label="Cancel" icon="pi pi-times" className={'p-button-secondary p-button-text'}
                                onClick={() => props.onResolve()}/>
                        <Button label="Create" icon="pi pi-check" onClick={() => props.onResolve({
                            collectionName,
                            collectionPath,
                            importCollectionFilepath
                        })}
                                style={{marginLeft: '80px'}}/>
                    </div>
                }>
            <div style={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'flex-start',
                marginBottom: '50px',
                marginTop: '40px'
            }}>
                <h3 style={{marginTop: '30px', marginBottom: '20px'}}>Collection File</h3>
                <div style={{display: 'flex', width: '100%'}}>
                    <Button label={"Choose"}
                            onClick={importCollectionFilePicker} style={{}}/>
                    <InputText autoFocus={true} value={importCollectionFilepath} style={{flexGrow: 1, marginLeft: '20px'}}
                               disabled={true}/>
                </div>

                <h3 style={{marginTop: '30px', marginBottom: '20px'}}>Target Location</h3>
                <p style={{}}>Where do you want to store the imported collection?</p>
                <div style={{marginTop: '10px', display: 'flex', width: '100%'}}>
                    <Button label={"Choose Empty Folder"}
                            onClick={openCollectionDirectoryPicker} style={{}}/>
                    <InputText value={collectionPath} style={{flexGrow: 1, marginLeft: '20px'}} disabled={true}/>
                </div>

            </div>

        </Dialog>
    )
}
