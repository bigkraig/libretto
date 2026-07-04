import {ITreeNode} from "@/lib/api/types";
import {GetTreeNodes} from "@/lib/api/GetTreeNodes";
import {HREF} from "@/lib";

export interface INavigatorLink {
  text: string
  location: string;
  selected: boolean | undefined
  icon: string | undefined
  kind: string
  href: string
}

export class NavigatorLink {
  text: string;
  location: string;
  selected: boolean | undefined;
  icon: string | undefined;
  kind: string;
  href: string;

  constructor(config: INavigatorLink) {
    this.text = config.text;
    this.location = config.location;
    this.selected = config.selected;
    this.icon = config.icon;
    this.kind = config.kind;
    this.href = config.href;
  }

  // contains logic to determine if child node is active
  static FromChildNode(config: ITreeNode, activeNode: ITreeNode | null): NavigatorLink {
    let link_node_id = config.node_value == activeNode?.node_value ? activeNode.parent_node_id : config.node_id;

    return new NavigatorLink({
      href: HREF(config.vehicle, config.year, link_node_id),
      location: config.location,
      icon: undefined,
      kind: config.isDriveFile() ? "drive_file" : "folder",
      selected: config.node_value == activeNode?.node_value,
      text: config.node_value ? config.node_value + " " + config.name : config.name
    })
  }

  // FromDriveFileTreeNode is different since we render its parents tree but with it selected
  static async FromDriveFileTreeNode(activeNode: ITreeNode): Promise<NavigatorLink[]> {
    let parent = await GetTreeNodes(activeNode.vehicle, activeNode.year, activeNode.parent_node_id);
    return NavigatorLink.FromTreeNode(parent, activeNode);
  }

  static async FromTreeNode(config: ITreeNode, activeNode: ITreeNode | null = null): Promise<NavigatorLink[]> {
    // we get the parent when its a drive file
    if (config.isDriveFile()) {
      return NavigatorLink.FromDriveFileTreeNode(config);
    }

    // Going "up" from a folder lands on its parent; from the vehicle root ("000")
    // there is no parent folder, so up leaves to the vehicle selection list.
    const isRoot = config.node_value == "000";
    let href = isRoot ? "/" : HREF(config.vehicle, config.year, config.parent_node_id)
    let children = config.children ? config.children.map((c) => NavigatorLink.FromChildNode(c, activeNode)) : [];

    return [new NavigatorLink({
      href,
      location: config.location,
      icon: undefined,
      kind: config.isDriveFile() ? "drive_file" : "open_folder",
      selected: false,
      text: config.node_value == "000" ? config.name : config.node_value + " " + config.name
    }), ...children]
  }
}