import { RequestModel } from "../../bindings"
import { RelynxState, useRequestModelStore } from "../../stores/requestStore"
import { Filepicker } from "../Filepicker"

interface ComponentProps {
  path: string,
  updatePath: (newPath: string) => void
}

export function BinaryFileComp(props: ComponentProps) {
  const currentRequest = useRequestModelStore((state: RelynxState) => state.currentRequest as RequestModel);

  return (
    <div style={{display: 'flex', flexDirection: 'column', alignItems: 'flex-start'}}>
      <h2>Binary File</h2>
      <Filepicker style={{marginTop: '20px'}} path={props.path} updatePath={props.updatePath} relativeBase={currentRequest.rest_file_path} />
    </div>
  )
}
