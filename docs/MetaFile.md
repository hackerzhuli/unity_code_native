## Meta File
every asset in a Unity project have a corresponding meta file, the meta file is a yaml file, it contains some meta information about the asset, guid is one of the key info contained in it.

eg. if there is a file in this path in project `Assets/Textures/Image.png`, then there must be a `.meta` file for it at `Assets/Textures/Image.png.meta`.

## Importers
Except for a few fields that are general, the actual content of meta file depends on the type of the asset(which is determined by its extension).

There are few types of notable importers(which determined the actual content of the .meta file) that is relevant:

1. Some asset doesn't have importer at all, eg. a `.cs` source file, no importer just default fields.
2. Default importer. some assets use default importer, eg. `.css` or a folder, unity don't actually use these as real assets, because they are not useful in Unity project.
3. Texture importer, which is for supported texture format, eg. `.png` and other texture that are natively supported by Unity.
4. Scripted importer, which is more general, it is a custom imported defined by a `.cs` script. eg. `.uss`, `.uxml` file actually use these importers.
   
When we need to get subassets from an asset, we only need to care about the Texture importer, because that is the only type that have subassets that is relevant to our project.

Because a texture can contain many sprites, we need to get the actual sprites data so that we can help with auto completion.

## File ID of an asset
In Unity when we refrence an asset, we need to have a guid, a file id and a type.

guid is the id of the asset. fild id is the id of a subasset in the file, if there are subassets in the file. Type is not documented we will assume it is always 3.

special file id: in most cases, for most assets that doesn't have subassets, the file id is a fixed number: `7433441132597879392`, we will assume this for now.

For textures that have sprites, the file id (typically called `internalID` inside of the `.meta` file) of sprites should be read from the `.meta` file `TextureImporter` part.

example of an asset that doesn't have subassets, a uxml file:

``` yaml
fileFormatVersion: 2
guid: 990f791f0aee3f04e8e9eba2ff279777
ScriptedImporter:
  internalIDToNameTable: []
  externalObjects: {}
  serializedVersion: 2
  userData: 
  assetBundleName: 
  assetBundleVariant: 
  script: {fileID: 13804, guid: 0000000000000000e000000000000000, type: 0}
```

The only interesting part is the guid, everything else can be ignored.

## Multiple Sprites in Texture Importer
Example of an `.png` texture that have multiple sprites(some lines are removed because the file is too long, and some fields are not relevant):

``` yaml
fileFormatVersion: 2
guid: 6a1cda2d4d23f0f43ab961e7dde2bd4a
TextureImporter:
  spriteMode: 2
  textureType: 8
  spriteSheet:
    nameFileIdTable:
      Hover Doc Link_0: -1713611897823765776
      Hover Doc Link_1: -970562782
      Hover Doc Link_2: -577418574
```

It is obvious that this field `nameFileIdTable` is the mapping of the sprite name to the file id, is the thing we need for our auto completion. The Id is the file id of the sprite, the name is the name of the sprite, for human readable purpose only, the id is the important one.

Note that we must confirm that the texture can have multiple sprites by checking `textureType` is 8 and `spriteMode` is 2, only then we can have multiple sprites in the texture, otherwise, the texture is the same as other assets, ie. there are no subassets.

Note, if no texture imported in present in the .meta file, we can just assume that it is a type of asset that doesn't have subassets(at least for our project, this can be assumed, because we only deal with a few types of assets).