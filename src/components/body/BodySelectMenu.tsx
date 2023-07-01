import { useEffect, useState } from "react";
import { TreeSelect } from 'primereact/treeselect';
import { newUUID } from "../../model/request";

export type BodyType = "plain_text" | "json" | "xml" | "yaml" | "other" | "no_body" | "form_urlencoded" | "multipart_form" | "graphql" | "binary_file";



export const BodyTypes: { plain_text: BodyType, json: BodyType, xml: BodyType, yaml: BodyType, other: BodyType, no_body: BodyType, form_urlencoded: BodyType, multipart_form: BodyType, binary_file: BodyType } = {
  plain_text: "plain_text",
  json: "json",
  xml: "xml",
  yaml: "yaml",
  other: "other",
  no_body: "no_body",
  form_urlencoded: "form_urlencoded",
  multipart_form: "multipart_form",
  binary_file: "binary_file",
  //@TODO: graphql: "graphql"
}

const mimeTypeMap: Map<BodyType, string> = new Map([
  [BodyTypes.plain_text, "text/plain"],
  [BodyTypes.json, "application/json"],
  [BodyTypes.xml, "application/xml"],
  [BodyTypes.yaml, "text/yaml"],
  [BodyTypes.other, ""],
  [BodyTypes.no_body, ""],
  [BodyTypes.form_urlencoded, "application/x-www-form-urlencoded"],
  [BodyTypes.multipart_form, "multipart/form-data"],
  [BodyTypes.binary_file, "application/octet-stream"],
  //@TODO[BodyTypes.graphql, "application/json"]
]);

export function toMimeType(bodyType: BodyType): string | undefined {
  return mimeTypeMap.get(bodyType);
}

export const TextBodyTypes: BodyType[] = [
  BodyTypes.plain_text,
  BodyTypes.json,
  BodyTypes.xml,
  BodyTypes.yaml,
  BodyTypes.other,
];

export interface BodyTypeNodeData {
  type: BodyType,
  isText: boolean
}


// https://primereact.org/treeselect/#api.TreeNode.props
const menuNodes = [
  {
    key: 'text',
    selectable: false,
    label: 'Text',
    children: [
      {
        key: BodyTypes.plain_text,
        selectable: true,
        label: 'Plain Text',
        data: { type: BodyTypes.plain_text, isText: true }
      },
      {
        key: BodyTypes.json,
        selectable: true,
        label: 'JSON',
        data: { type: BodyTypes.json, isText: true },
      },
      {
        key: BodyTypes.xml,
        selectable: true,
        label: 'XML',
        data: { type: BodyTypes.xml, isText: true },
      },
      {
        key: BodyTypes.yaml,
        selectable: true,
        label: 'YAML',
        data: { type: BodyTypes.yaml, isText: true }
      },

      {
        key: BodyTypes.other,
        selectable: true,
        label: 'Other',
        data: { type: BodyTypes.other, isText: true },
      },
    ]
  },
  {
    key: BodyTypes.form_urlencoded,
    selectable: true,
    label: 'Form Urlencoded',
    data: { type: BodyTypes.form_urlencoded, isText: false },
  },
  {
    key: BodyTypes.multipart_form,
    selectable: true,
    label: 'Multipart form',
    data: { type: BodyTypes.multipart_form, isText: false },
  },
  //@TODO {
  //   key: '9',
  //   selectable: true,
  //   label: 'GraphQL',
  //   data: 'graphql'
  // },

  {
    key: BodyTypes.binary_file,
    selectable: true,
    label: 'Binary File',
    data: { type: BodyTypes.binary_file, isText: false },
    children: [
    ]
  },
  {
    key: "no_body",
    selectable: true,
    label: 'No Body',
    data: { type: 'no_body', isText: false },
  },

];

interface ComponentProps {
  currentType: BodyType,
  setNewType: (bodyType: BodyType, isText: boolean) => void,
  style: any
}

export default function BodySelectMenu(props: ComponentProps) {
  const [nodes, setNodes] = useState(menuNodes);
  const [selectedNodeKey, setSelectedNodeKey] = useState<string | undefined>(undefined);
  const [expandedKeys, setExpandedKeys] = useState({});

  const expandAll = () => {
    let _expandedKeys = {};

    for (let node of nodes) {
      expandNode(node, _expandedKeys);
    }

    setExpandedKeys(_expandedKeys);
  };

  useEffect(() => {
    expandAll();
    setSelectedNodeKey(props.currentType);
  }, [props.currentType])

  const collapseAll = () => {
    setExpandedKeys({});
  };

  const expandNode = (node: any, _expandedKeys: any) => {
    if (node.children && node.children.length) {
      _expandedKeys[node.key] = true;

      for (let child of node.children) {
        expandNode(child, _expandedKeys);
      }
    }
  };

  const onNodeSelect = (value: { node: any }) => {
    props.setNewType(value.node.data.type, value.node.data.isText);
  }

  return (
    <div className="card flex justify-content-center" style={{ minWidth: '200px', ...(props.style ?? {}) }}>
      <TreeSelect value={selectedNodeKey} onNodeSelect={onNodeSelect} onChange={(e: any) => setSelectedNodeKey(e.value)} options={nodes}
        style={{ width: '100%' }} placeholder="Select Bodytype"
        expandedKeys={expandedKeys} onToggle={(e) => setExpandedKeys(e.value)} ></TreeSelect>
    </div>
  );
}
