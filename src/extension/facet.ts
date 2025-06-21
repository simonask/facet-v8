declare const __boxBrand: unique symbol;
interface BoxPtr {
  readonly [__boxBrand]: never;
}
declare const __shapeBrand: unique symbol;
interface ShapePtr {
  readonly [__shapeBrand]: never;
}
declare const __valueVTableBrand: unique symbol;
interface ValueVTablePtr {
  readonly [__valueVTableBrand]: never;
}

declare const Deno: {
  core: {
    ops: {
      op_facet_value_vtable_type_name: (ptr: ValueVTablePtr) => string;
      op_facet_value_vtable_marker_traits: (ptr: ValueVTablePtr) => number;
    };
  };
};

const FRIEND = Symbol("facet:friend");

export interface Shape {
  get id(): ConstTypeId;
  get layout(): Layout | UnsizedType;
  get vtable(): ValueVTable;
  get def(): Def;
  get typeIdentifier(): string;
  get typeParams(): TypeParam[];
  get doc(): string[];
  //   get attributes(): ShapeAttribute[];
  get typeTag(): string | null;
  get inner(): Shape | null;
}

export interface ConstTypeId {
  isEqual: (other: ConstTypeId) => boolean;
}

export const Unsized: unique symbol = Symbol("Unsized");
export type UnsizedType = typeof Unsized;

export interface Layout {
  get size(): number;
  get align(): number;
}

export class ValueVTable {
  #ptr: ValueVTablePtr;
  get typeName(): string {
    return Deno.core.ops.op_facet_value_vtable_type_name(this.#ptr);
  }
  get markerTraits(): MarkerTraits {
    return Deno.core.ops.op_facet_value_vtable_marker_traits(this.#ptr);
  }
}

export enum MarkerTraits {
  Eq = 1,
  Send = 1 << 1,
  Sync = 1 << 2,
  Copy = 1 << 3,
  Unpin = 1 << 4,
  UnwindSafe = 1 << 5,
  RefUnwindSafe = 1 << 6,
}

export abstract class Def {
  isArray = (): boolean => this instanceof ArrayDef;
}

export class ScalarDef extends Def {}
export class MapDef extends Def {}
export class SetDef extends Def {}
export class ListDef extends Def {}
export class ArrayDef extends Def {}
export class SliceDef extends Def {}
export class OptionDef extends Def {}
export class SmartPointerDef extends Def {}

export class TypeParam {
  readonly name: string;
  readonly shape: Shape;
}

export interface PtrUninit {
  readonly shape: Shape;
}

export interface PtrConst {
  readonly shape: Shape;
  readonly invariants: boolean;
  readonly display: string | null;
  readonly debug: string | null;
  readonly cloneInto: ((target: PtrUninit) => void) | null;
  readonly partialEq: ((other: PtrConst) => boolean) | null;
  readonly partialOrd: ((other: PtrConst) => number) | null;
  readonly ord: ((other: PtrConst) => number) | null;
  readonly hash:
    | ((state: { write: (data: Uint8Array) => void }) => void)
    | null;
}

export interface PtrMut extends PtrConst {}

export interface Box extends PtrMut {}
