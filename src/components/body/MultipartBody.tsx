import { Button } from "primereact/button"
import { InputText } from "primereact/inputtext"
import { Multipart } from "../../bindings"
import { RequestBodyMultipart } from "../../model/request"
import { SingleMultipart } from "./SingleMultipart"

interface ComponentProps {
  body: RequestBodyMultipart,
  addPart: () => void
  updateBodyMultipart: (newBody: RequestBodyMultipart) => void,
}

export function MultipartBody(props: ComponentProps) {
  const updateBoundary = (newBoundary: string) => {
    let newBody = structuredClone(props.body);
    newBody.Multipart.boundary = newBoundary;
    props.updateBodyMultipart(newBody);
  }

  const removeMultipart = (index: number) => {
    let newBody = structuredClone(props.body);
    newBody.Multipart.parts.splice(index, 1);
    props.updateBodyMultipart(newBody);
  }

  const updatePart = (index: number, newPart: Multipart) => {
    let newBody = structuredClone(props.body);
    newBody.Multipart.parts[index] = newPart;
    props.updateBodyMultipart(newBody);
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', width: '100%' }}>
      <div><span>Boundary: </span><InputText
        value={props.body.Multipart.boundary}
        onChange={(event) => updateBoundary(event.target.value)}
        style={{ marginLeft: '20px' }}
      /></div>
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', width: '100%' }}>
        {
          props.body.Multipart.parts.map((part: Multipart, index: number) => {
            return <SingleMultipart updatePart={(newPart: Multipart) => { updatePart(index, newPart) }} part={part} style={{ marginTop: '20px' }} onRemove={() => removeMultipart(index)} />
          })
        }
        <Button label="Add Part" icon="pi pi-plus"
          onClick={props.addPart}
          style={{ marginTop: '40px' }} />

      </div>
    </div>
  )
}
