import { Accordion, AccordionTab } from "primereact/accordion";
import { Button } from "primereact/button";
import { confirmPopup } from "primereact/confirmpopup";
import { Dropdown } from "primereact/dropdown";
import { InputText } from "primereact/inputtext"
import { InputTextarea } from "primereact/inputtextarea";
import { useEffect, useState } from "react";
import { Header, Multipart, RequestModel } from "../../bindings"
import { catchError } from "../../common/errorhandling";
import { DataSourceFromFilepath, DataSourceRaw } from "../../model/request";
import { backend } from "../../rpc";
import { RelynxState, useRequestModelStore } from "../../stores/requestStore";
import { CopyToClipboard } from "../CopyToClipboard";
import { KeyValueRow } from "../KeyValueRow";

interface ComponentProps {
  part: Multipart,
  style: any,
  onRemove: () => void,
  updatePart: (newPart: Multipart) => void
}

const optionRaw = { name: 'Raw Text', key: 'text_raw' };
const optionFromFile = { name: 'From File', key: 'from_file' };

export function SingleMultipart(props: ComponentProps) {


  const currentEnvironment = useRequestModelStore((state: RelynxState) => state.currentEnvironment);
  const currentRequest = useRequestModelStore((state: RelynxState) => state.currentRequest as RequestModel);

  const [dataRaw, setDataRaw] = useState<string>("");
  const [filePath, setFilePath] = useState<string>("");

  useEffect(() => {
    if ((props.part.data as DataSourceRaw<string>).Raw !== undefined) {
      setDataRaw((props.part.data as DataSourceRaw<string>).Raw);
    } else {
      setFilePath((props.part.data as DataSourceFromFilepath).FromFilepath);
    }
  }, [props.part])

  const confirmRemove = (event: any) => {
    confirmPopup({
      target: event.currentTarget,
      message: 'Are you sure you want to remove this part?',
      icon: 'pi pi-exclamation-triangle',
      accept: props.onRemove,
      reject: () => {
      },
    });
    event.stopImmediatePropagation();
    event.preventDefault();
  };

  const updateHeaderKey = (index: number, newKey: string) => {
    let newPart = structuredClone(props.part);
    newPart.headers[index].key = newKey;
    props.updatePart(newPart);

  }
  const updateHeaderValue = (index: number, newValue: string) => {
    let newPart = structuredClone(props.part);
    newPart.headers[index].value = newValue;
    props.updatePart(newPart);
  }

  const addHeader = () => {
    let newPart = structuredClone(props.part);
    newPart.headers.push({ key: "", value: "", active: true });
    props.updatePart(newPart);
  };

  const removeHeader = (index: number) => {
    let newPart = structuredClone(props.part);
    newPart.headers.splice(index, 1);
    props.updatePart(newPart);
  }

  const chooseMultipartFile = () => {
    backend.chooseFileRelativeTo(currentRequest?.rest_file_path).then((file: string) => {
      updateFromFilepath(file);
    }).catch(catchError);
  }

  const updateDataSource = (event: any) => {
    let newPart = structuredClone(props.part);
    if (event.value.key == 'text_raw') {
      newPart.data = { Raw: dataRaw };
    } else {
      newPart.data = { FromFilepath: filePath };
    }
    props.updatePart(newPart);
  }

  const updateDataRaw = (newData: string) => {
    let newPart = structuredClone(props.part);
    (newPart.data as DataSourceRaw<string>) = { Raw: newData };
    props.updatePart(newPart);
  }

  const updateFromFilepath = (newPath: string) => {
    let newPart = structuredClone(props.part);
    (newPart.data as DataSourceFromFilepath) = { FromFilepath: newPath };
    props.updatePart(newPart);
  }

  return (
    <Accordion style={{ width: '100%', ...(props.style ?? {}) }} activeIndex={0}>
      <AccordionTab header={"Multipart"} tabIndex={0}>
        <Button size="small" text={true} className="p-button-danger" style={{ position: 'absolute', right: '5px', top: '5px' }} onClick={(event: any) => confirmRemove(event)} icon={"pi pi-trash"} />
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
          <div style={{ display: 'flex', alignItems: 'center', width: '100%', justifyContent: 'flex-start' }}>
            <label style={{ flexBasis: '15%', textAlign: 'start' }}>Name: </label>
            <InputText style={{ marginLeft: '20px' }} value={props.part.disposition.name} />
          </div>
          <div style={{ display: 'flex', alignItems: 'center', width: '100%', justifyContent: 'flex-start', marginTop: '20px' }}>
            <label style={{ flexBasis: '15%', textAlign: 'start' }}>Filename: </label>
            <InputText style={{ marginLeft: '20px' }} value={props.part.disposition.filename ?? ""} />
          </div>
          {/* <div style={{ display: 'flex', alignItems: 'center', width: '100%', justifyContent: 'flex-start', marginTop: '20px'}}> */}
          {/*   <label style={{flexBasis: '15%', textAlign: 'start'}}>Filename*: </label> */}
          {/*   <InputText style={{ marginLeft: '20px' }} value={props.part.disposition.filename_star ?? ""} /> */}
          {/* </div> */}

          {props.part.headers.length > 0 &&
            <div style={{ marginTop: '40px', display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
              <h3>Headers</h3>
              <div style={{ marginTop: '22px' }}>
                {props.part.headers.map((header: Header, index: number) => {
                  return <KeyValueRow key={index} keyProperty={header.key}
                    valueProperty={header.value}
                    active={header.active}
                    keyLabel={"Header Name"} valueLabel={"Header Value"}
                    updateKey={(key: string) => updateHeaderKey(index, key)}
                    updateValue={(value: string) => updateHeaderValue(index, value)}
                    updateActive={undefined}
                    remove={() => removeHeader(index)}
                    style={{ marginTop: '20px' }}
                    currentEnvironment={currentEnvironment}
                    withHeader={index == 0 ? { keyHeader: 'Name', valueHeader: "Value" } : undefined}
                  />
                })}
              </div>
            </div>
          }

          <Dropdown onChange={updateDataSource} style={{ marginTop: '20px' }} optionLabel={"name"} options={[optionRaw, optionFromFile]} value={
            ((props.part.data as DataSourceRaw<string>).Raw !== undefined) ? optionRaw : optionFromFile
          } />

          {
            (props.part.data as DataSourceRaw<string>).Raw !== undefined &&
            <div style={{ marginTop: '30px', display: 'flex', flexDirection: 'column', alignItems: 'flex-start', width: '100%' }}>
              <h3>Data</h3>
              <InputTextarea onChange={(event: any) => updateDataRaw(event.target.value)} style={{ marginTop: '20px', width: '100%', minHeight: '300px' }} value={dataRaw} />
            </div>
          }

          {
            (props.part.data as DataSourceFromFilepath).FromFilepath !== undefined &&
            <div style={{ marginTop: '20px', display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
              <h3>File</h3>
              <div style={{ display: 'flex', alignItems: 'center', width: '100%', justifyContent: 'flex-start', marginTop: '20px' }}>
                <label style={{ flexBasis: '15%', textAlign: 'start' }}>Path (relative to request): </label>
                <InputText onChange={(event: any) => updateFromFilepath(event.target.value)} disabled={true} style={{ marginLeft: '20px' }} value={filePath} />
                {
                  filePath !== "" && <CopyToClipboard value={filePath} />
                }
              </div>
              <Button style={{ marginTop: '20px' }} text={true} label={"Choose File"} onClick={chooseMultipartFile} />
            </div>
          }

          <div style={{ display: 'flex', alignItems: 'center', marginTop: '30px', width: '100%', justifyContent: 'space-between' }}>
            <Button icon={"pi pi-plus"} onClick={addHeader} label={"Add Header"} style={{}} />
            <Button size="small" style={{ marginLeft: '20px' }} icon={"pi pi-trash"} onClick={(event: any) => confirmRemove(event)} className={"p-button-danger"} />
          </div>
        </div>
      </AccordionTab>
    </Accordion>
  )
}
