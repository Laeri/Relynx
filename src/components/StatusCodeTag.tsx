import {Tag} from "primereact/tag";
import {getStatusTextForCode} from "../model/request";

export interface ComponentProps {
    statusCode: string,
    style: any
}


export type TagSeverityType =   'success' | 'info' | 'warning' | 'danger' | null | undefined;
export function StatusCodeTag(props: ComponentProps) {
    let severity: TagSeverityType = "info";
    // status 2XX is success, 4XX or 5XX something went wrong
    if (props.statusCode?.startsWith('2')) {
        severity = "success";
    } else if (props.statusCode?.startsWith('4') || props.statusCode?.startsWith('5')) {
        severity = "danger";
    }

    let statusCodeText = getStatusTextForCode(props.statusCode);
    let styles = props.style ?? {};
    return (<>
            {props.statusCode && <Tag value={`Status: ${props.statusCode} ${statusCodeText}`} severity={severity}
                                      style={{...styles, maxHeight: '25px'}}/>}
        </>
    )
}
