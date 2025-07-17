# CS Docs
## CS Doc Assembly
The goal is to build xml docs assemblies just like C# compiler builds real assemblies.

If C# compiler creates a file called `Hello.World.dll`, then we will create a file called `Hello.World.json`, which contains xml docs in for that assembly.

This also means we can watch the compiled dlls and look for changes, if it changed, then we will also recreate the corrresponding json file from source files.

That does not mean we will eagerly create a new json file when ever a dll changed, just take notes and create json file when needed.

There are two different locations of source code, type one is user code, outside of the `Library` directory, either in `Assets` or `Packages`. The other location of source code, is from package cache, living inside of `Library/PackageCache` directory.

## Finding assembly sources from package cache
There is one core file we need to read from, that is the `Packages/packages-lock.json` file inside of the Unity project. Within it, we will find all packages basic info that is use by this project.

Once we have the package names, and their versions, then we look at the `Library/PackageCache` directory to look for the actual package. 

Each package is a directory inside of that `Library/PackageCache` directory. 

eg. `com.unity.2d.animation@494a3b4e73a9`, this is the directory that contains the package `com.unity.2d.animation`.

Note that there is an additional `@` character and a hash in the directory name.

Within that directory, we will find a `package.json` file, which contains the package info.

example `package.json` file(with irrelevant info removed)
``` json
{
  "name": "com.unity.2d.animation",
  "version": "10.1.4",
  "unity": "2023.1",
  "displayName": "2D Animation",
  "description": "2D Animation provides all the necessary tooling and runtime components for skeletal animation using Sprites.",
  "documentationUrl": "https://docs.unity3d.com/Packages/com.unity.2d.animation@10.1/manual/index.html",
  "repository": {
    "url": "https://github.cds.internal.unity3d.com/unity/2d.git",
    "type": "git",
    "revision": "c393ad93bdba3e78ebd30a5ccd2e6da5d8b92aba"
  },
}
```

Also within the package's directory, we are looking for top level directories that contains an `.asmdef` file.

Eg. within `Library/PackageCache/com.unity.2d.animation@494a3b4e73a9/Runtime`, we found `Unity.2D.Animation.Runtime.asmdef` file, here is its content, it's json, which irrelevant info removed.
``` json
{
    "name": "Unity.2D.Animation.Runtime",
    "rootNamespace": "",
    "includePlatforms": [],
    "excludePlatforms": [],
    "allowUnsafeCode": true,
    "overrideReferences": false,
    "precompiledReferences": [],
    "autoReferenced": true,
    "defineConstraints": [],
    "noEngineReferences": false
}
```

The name field is the key, eg. the name in the above file means source code in this `Runtime` directory, will be used to compile `Unity.2D.Animation.Runtime.dll` file, as the name field indicated(not the name of the `.asmdef` file, but the name field that is defined in it).

Then we will assume that every `.cs` file inside of that `Runtime` directory, will be used to compile `Unity.2D.Animation.Runtime.dll` file, which is typically how Unity packages are organized.

We will go ahead and compile xml docs into a single json file, just like how C# compiler compiles the source code to an assembly.

Note:
- Each package can only have one directory inside of `Library/PackageCache` directory, if user updated a package, the older version will be removed from the directory. So we can assume there is only one directory for one package.

## Finding source from user code
User code don't live in `Library/PackageCache` directory, they live in `Assets` or `Packages` directory.

Unity will generate C# project files for them in project root.

eg. `Assembly-CSharp.csproj` file, means this is for the assebmly `Assembly-CSharp.dll`.

example content, it's xml with irrelevant info removed:
``` xml
<Project>
  <Import Project="Sdk.props" Sdk="Microsoft.NET.Sdk" />
  <ItemGroup>
    <ProjectCapability Include="Unity" />
  </ItemGroup>
  <PropertyGroup>
    <GenerateAssemblyInfo>false</GenerateAssemblyInfo>
    <EnableDefaultItems>false</EnableDefaultItems>
    <LangVersion>9.0</LangVersion>
    <RootNamespace></RootNamespace>
    <OutputType>Library</OutputType>
    <AssemblyName>Assembly-CSharp</AssemblyName>
    <TargetFramework>netstandard2.1</TargetFramework>
    <BaseDirectory>.</BaseDirectory>
  </PropertyGroup>
  <PropertyGroup>
    <UnityProjectGenerator>Package</UnityProjectGenerator>
    <UnityProjectGeneratorVersion>1.0.4</UnityProjectGeneratorVersion>
    <UnityProjectGeneratorStyle>SDK</UnityProjectGeneratorStyle>
    <UnityProjectType>Game:1</UnityProjectType>
    <UnityBuildTarget>StandaloneWindows64:19</UnityBuildTarget>
    <UnityVersion>6000.0.51f1</UnityVersion>
  </PropertyGroup>
  <ItemGroup>
    <Compile Include="Assets\Scripts\Anim.cs" />
    <Compile Include="Assets\Scripts\TestAnalyzer.cs" />
    <Compile Include="Assets\Scripts\Script2.cs" />
    <Compile Include="Assets\Scripts\MyElement.cs" />
    <Compile Include="Assets\Scripts\TestHover.cs" />
  </ItemGroup>
</Project>
```

Note that we have an `ItemGroup` that has `Compile` tags in it, they are the relative path for source code.

Also note that there is an tag `AssemblyName`, it's value is `Assembly-CSharp`, which means the source code files will be compiled to `Assembly-CSharp.dll` file.

That's how we find the source files for user code.

## Compile xml docs for assembly
Now we know how to find source files.

Next step is to parse all files and extract xml docs for them.

Once we gather all xml docs for an assembly, we create a `.json` file to store all xml docs for that assembly in a `.json` with the same name of the assembly inside of the `Library/UnityCode/DocAssemblies` directory.

## Watching the assemblies
Unity puts all compiled assemblies, including from `Library/PackageCache`, and user code, in a directory `Library/ScriptAssemblies`

If any `.dll` file changed, that means Unity has compiled it, and we will have to update our corresponding `.json` file when needed.

## Watching the package lock file
If any `package-lock.json` file changed, that means Unity has updated it, and we will need to rescan packages in package cache.

User might have updated packages, or added or removed packages.

That does not mean we will compile our docs files(which should only need to update when the corresponding `.dll` files changed). It just means we need to update where the source location for assemblies are when needed.

## Content of the `.json` docs assembly
We have a limitation, we only extract top level types, if there are nested types, we don't extract them.

For non user code, ie. code from `Library/PackageCache`, we only extract xml docs for public types and members, to make our output smaller and more efficient.

For user code, ie. source files that is defined in a `.csproj` file in Unity project root, we will extract xml docs for all top level types and their members, even private types and members.

Note that for methods, we need to store the the parameter types for that method as part of the member name, because methods can have overloads, so we must have a way to distinguish between them.

The data should be orgnized by types, each type is named fully qualified name, and all members' data of that type is stored in that type.

For methods, names must add the parameter types as part of the name(and generic parameter if exists), as stated above.

Other types of members just the name is enough.

